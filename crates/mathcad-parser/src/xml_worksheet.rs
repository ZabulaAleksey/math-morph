use std::collections::HashSet;
use std::str;
use std::sync::Arc;

use quick_xml::events::{BytesStart, Event};
use quick_xml::name::ResolveResult;
use quick_xml::{NsReader, XmlVersion};

use crate::{
    CoordinateError, CustomValueKind, Diagnostic, DiagnosticCode, ExpandedName, InlineAttribute,
    InlineKind, MathParseOutcome, MathRegion, OpaqueFragment, OpaqueTableResult, PictureKind,
    PictureRegion, PlotRegion, Region, RegionContent, RegionLayout, ResultFormat, SourceDocument,
    SourceNumber, SourceSpan, TextParagraph, TextRegion, TextRun, TextValue, Worksheet,
    WorksheetCustomValue, WorksheetError, WorksheetIdentityInfo, WorksheetLimit, WorksheetLimits,
    WorksheetMetadata, WorksheetUserData,
};

const WS_NS: &str = "http://schemas.mathsoft.com/worksheet30";
const MATH_NS: &str = "http://schemas.mathsoft.com/math30";
const SUPPORTED_VERSION: &str = "3.0.3";

#[derive(Debug)]
struct Attribute {
    name: ExpandedName,
    value: String,
}

#[derive(Debug)]
pub(crate) enum Child {
    Node(Node),
    Text { value: String, span: SourceSpan },
}

#[derive(Debug)]
pub(crate) struct Node {
    pub(crate) name: ExpandedName,
    attributes: Vec<Attribute>,
    pub(crate) children: Vec<Child>,
    pub(crate) span: SourceSpan,
}

impl Node {
    pub(crate) fn is(&self, namespace: &str, local: &str) -> bool {
        self.name.namespace_uri.as_deref() == Some(namespace) && self.name.local_name == local
    }

    pub(crate) fn attribute(&self, local: &str) -> Option<&str> {
        self.attributes
            .iter()
            .find(|attribute| {
                attribute.name.namespace_uri.is_none() && attribute.name.local_name == local
            })
            .map(|attribute| attribute.value.as_str())
    }

    fn child(&self, namespace: &str, local: &str) -> Option<&Node> {
        self.children.iter().find_map(|child| match child {
            Child::Node(node) if node.is(namespace, local) => Some(node),
            _ => None,
        })
    }

    pub(crate) fn element_children(&self) -> impl Iterator<Item = &Node> {
        self.children.iter().filter_map(|child| match child {
            Child::Node(node) => Some(node),
            Child::Text { .. } => None,
        })
    }

    fn text(&self) -> String {
        fn append(node: &Node, output: &mut String) {
            for child in &node.children {
                match child {
                    Child::Node(node) => append(node, output),
                    Child::Text { value, .. } => output.push_str(value),
                }
            }
        }
        let mut output = String::new();
        append(self, &mut output);
        output
    }

    fn opaque(&self) -> OpaqueFragment {
        OpaqueFragment {
            name: self.name.clone(),
            span: self.span,
        }
    }
}

struct TreeBuilder {
    limits: WorksheetLimits,
    node_count: usize,
    retained_text_bytes: usize,
    namespace_uris: HashSet<Arc<str>>,
}

impl TreeBuilder {
    fn parse(mut self, bytes: &[u8]) -> Result<Node, WorksheetError> {
        if bytes.len() > self.limits.max_input_bytes {
            return Err(WorksheetError::LimitExceeded(WorksheetLimit::InputBytes));
        }
        if bytes.starts_with(&[0xff, 0xfe]) || bytes.starts_with(&[0xfe, 0xff]) {
            return Err(WorksheetError::UnsupportedEncoding);
        }
        str::from_utf8(bytes).map_err(|_| WorksheetError::UnsupportedEncoding)?;

        let mut reader = NsReader::from_reader(bytes);
        reader
            .resolver_mut()
            .set_max_declarations_per_element(self.limits.max_namespace_declarations);
        let mut buffer = Vec::new();
        let mut stack: Vec<Node> = Vec::new();
        let mut root = None;

        loop {
            let event_start = reader.buffer_position() as usize;
            let (resolution, event) = reader
                .read_resolved_event_into(&mut buffer)
                .map_err(map_xml_error)?;
            let element_namespace = self.resolved_namespace(resolution)?;
            let event_end = reader.buffer_position() as usize;
            match event {
                Event::Decl(declaration) => {
                    if let Some(encoding) = declaration.encoding() {
                        let encoding = encoding.map_err(|_| WorksheetError::MalformedXml)?;
                        if !str::from_utf8(&encoding)
                            .map_err(|_| WorksheetError::UnsupportedEncoding)?
                            .eq_ignore_ascii_case("UTF-8")
                        {
                            return Err(WorksheetError::UnsupportedEncoding);
                        }
                    }
                }
                Event::DocType(_) => return Err(WorksheetError::DoctypeForbidden),
                Event::Start(start) => {
                    if stack.len() >= self.limits.max_xml_depth {
                        return Err(WorksheetError::LimitExceeded(WorksheetLimit::XmlDepth));
                    }
                    let node = self.start_node(
                        &reader,
                        element_namespace,
                        &start,
                        SourceSpan {
                            start: event_start,
                            end: event_end,
                        },
                    )?;
                    stack.push(node);
                }
                Event::Empty(start) => {
                    if stack.len() >= self.limits.max_xml_depth {
                        return Err(WorksheetError::LimitExceeded(WorksheetLimit::XmlDepth));
                    }
                    let node = self.start_node(
                        &reader,
                        element_namespace,
                        &start,
                        SourceSpan {
                            start: event_start,
                            end: event_end,
                        },
                    )?;
                    push_node(&mut stack, &mut root, node)?;
                }
                Event::End(_) => {
                    let mut node = stack.pop().ok_or(WorksheetError::MalformedXml)?;
                    node.span.end = event_end;
                    push_node(&mut stack, &mut root, node)?;
                }
                Event::Text(text) => {
                    let value = text
                        .xml_content(XmlVersion::Implicit1_0)
                        .map_err(|_| WorksheetError::MalformedXml)?
                        .into_owned();
                    self.push_text(
                        &mut stack,
                        value,
                        SourceSpan {
                            start: event_start,
                            end: event_end,
                        },
                    )?;
                }
                Event::CData(text) => {
                    let value = text
                        .xml_content(XmlVersion::Implicit1_0)
                        .map_err(|_| WorksheetError::MalformedXml)?
                        .into_owned();
                    self.push_text(
                        &mut stack,
                        value,
                        SourceSpan {
                            start: event_start,
                            end: event_end,
                        },
                    )?;
                }
                Event::GeneralRef(reference) => {
                    let name = reference
                        .decode()
                        .map_err(|_| WorksheetError::MalformedXml)?;
                    let escaped = format!("&{name};");
                    let value = quick_xml::escape::unescape(&escaped)
                        .map_err(|_| WorksheetError::MalformedXml)?
                        .into_owned();
                    self.push_text(
                        &mut stack,
                        value,
                        SourceSpan {
                            start: event_start,
                            end: event_end,
                        },
                    )?;
                }
                Event::Comment(_) | Event::PI(_) => {}
                Event::Eof => break,
            }
            buffer.clear();
        }
        if !stack.is_empty() {
            return Err(WorksheetError::MalformedXml);
        }
        root.ok_or(WorksheetError::MalformedXml)
    }

    fn start_node(
        &mut self,
        reader: &NsReader<&[u8]>,
        namespace_uri: Option<Arc<str>>,
        start: &BytesStart<'_>,
        span: SourceSpan,
    ) -> Result<Node, WorksheetError> {
        self.node_count = self
            .node_count
            .checked_add(1)
            .ok_or(WorksheetError::LimitExceeded(WorksheetLimit::XmlNodes))?;
        if self.node_count > self.limits.max_xml_nodes {
            return Err(WorksheetError::LimitExceeded(WorksheetLimit::XmlNodes));
        }
        validate_utf8_token(start.name().as_ref(), self.limits.max_token_bytes)?;
        let local = bounded_utf8(start.local_name().as_ref(), self.limits.max_token_bytes)?;
        let name = ExpandedName {
            namespace_uri,
            local_name: local,
        };
        let mut attributes = Vec::new();
        for raw_attribute in start.attributes() {
            let raw_attribute = raw_attribute.map_err(|_| WorksheetError::MalformedXml)?;
            validate_utf8_token(raw_attribute.key.as_ref(), self.limits.max_token_bytes)?;
            if raw_attribute.value.len() > self.limits.max_attribute_value_bytes {
                return Err(WorksheetError::LimitExceeded(
                    WorksheetLimit::AttributeValueBytes,
                ));
            }
            let value = raw_attribute
                .decoded_and_normalized_value(XmlVersion::Implicit1_0, reader.decoder())
                .map_err(|_| WorksheetError::MalformedXml)?
                .into_owned();
            if raw_attribute.key.as_namespace_binding().is_some() {
                continue;
            }
            if attributes.len() >= self.limits.max_attributes_per_element {
                return Err(WorksheetError::LimitExceeded(WorksheetLimit::Attributes));
            }
            let (namespace, local) = reader.resolver().resolve_attribute(raw_attribute.key);
            let local = bounded_utf8(local.as_ref(), self.limits.max_token_bytes)?;
            attributes.push(Attribute {
                name: ExpandedName {
                    namespace_uri: self.resolved_namespace(namespace)?,
                    local_name: local,
                },
                value,
            });
        }
        Ok(Node {
            name,
            attributes,
            children: Vec::new(),
            span,
        })
    }

    fn resolved_namespace(
        &mut self,
        resolution: ResolveResult<'_>,
    ) -> Result<Option<Arc<str>>, WorksheetError> {
        match resolution {
            ResolveResult::Bound(namespace) => {
                let namespace =
                    str::from_utf8(namespace.as_ref()).map_err(|_| WorksheetError::MalformedXml)?;
                if let Some(existing) = self.namespace_uris.get(namespace) {
                    return Ok(Some(Arc::clone(existing)));
                }
                let interned: Arc<str> = Arc::from(namespace);
                self.namespace_uris.insert(Arc::clone(&interned));
                Ok(Some(interned))
            }
            ResolveResult::Unbound => Ok(None),
            ResolveResult::Unknown(_) => Err(WorksheetError::UnknownNamespacePrefix),
        }
    }

    fn push_text(
        &mut self,
        stack: &mut [Node],
        value: String,
        span: SourceSpan,
    ) -> Result<(), WorksheetError> {
        self.node_count = self
            .node_count
            .checked_add(1)
            .ok_or(WorksheetError::LimitExceeded(WorksheetLimit::XmlNodes))?;
        if self.node_count > self.limits.max_xml_nodes {
            return Err(WorksheetError::LimitExceeded(WorksheetLimit::XmlNodes));
        }
        if value.len() > self.limits.max_token_bytes {
            return Err(WorksheetError::LimitExceeded(WorksheetLimit::TokenBytes));
        }
        self.retained_text_bytes = self.retained_text_bytes.checked_add(value.len()).ok_or(
            WorksheetError::LimitExceeded(WorksheetLimit::RetainedTextBytes),
        )?;
        if self.retained_text_bytes > self.limits.max_retained_text_bytes {
            return Err(WorksheetError::LimitExceeded(
                WorksheetLimit::RetainedTextBytes,
            ));
        }
        if let Some(parent) = stack.last_mut() {
            parent.children.push(Child::Text { value, span });
        } else if !value.trim().is_empty() {
            return Err(WorksheetError::MalformedXml);
        }
        Ok(())
    }
}

fn push_node(
    stack: &mut [Node],
    root: &mut Option<Node>,
    node: Node,
) -> Result<(), WorksheetError> {
    if let Some(parent) = stack.last_mut() {
        parent.children.push(Child::Node(node));
    } else if root.replace(node).is_some() {
        return Err(WorksheetError::MalformedXml);
    }
    Ok(())
}

fn bounded_utf8(bytes: &[u8], limit: usize) -> Result<String, WorksheetError> {
    Ok(validate_utf8_token(bytes, limit)?.to_owned())
}

fn validate_utf8_token(bytes: &[u8], limit: usize) -> Result<&str, WorksheetError> {
    if bytes.len() > limit {
        return Err(WorksheetError::LimitExceeded(WorksheetLimit::TokenBytes));
    }
    str::from_utf8(bytes).map_err(|_| WorksheetError::MalformedXml)
}

fn map_xml_error(error: quick_xml::Error) -> WorksheetError {
    match error {
        quick_xml::Error::Namespace(quick_xml::name::NamespaceError::TooManyDeclarations(_)) => {
            WorksheetError::LimitExceeded(WorksheetLimit::NamespaceDeclarations)
        }
        quick_xml::Error::Namespace(_) => WorksheetError::UnknownNamespacePrefix,
        quick_xml::Error::Encoding(_) => WorksheetError::UnsupportedEncoding,
        _ => WorksheetError::MalformedXml,
    }
}

pub(crate) fn parse_worksheet(
    bytes: &[u8],
    limits: WorksheetLimits,
) -> Result<Worksheet, WorksheetError> {
    let root = TreeBuilder {
        limits,
        node_count: 0,
        retained_text_bytes: 0,
        namespace_uris: HashSet::new(),
    }
    .parse(bytes)?;
    if !root.is(WS_NS, "worksheet") {
        return Err(WorksheetError::UnsupportedRoot);
    }
    let version = root
        .attribute("version")
        .ok_or(WorksheetError::UnsupportedVersion)?;
    if version != SUPPORTED_VERSION {
        return Err(WorksheetError::UnsupportedVersion);
    }

    let metadata = root
        .child(WS_NS, "metadata")
        .map(parse_metadata)
        .transpose()?;
    let mut regions = Vec::new();
    let mut diagnostics = Vec::new();
    let mut identifiers = HashSet::new();
    if let Some(region_container) = root.child(WS_NS, "regions") {
        discover_regions(
            region_container,
            limits,
            &mut identifiers,
            &mut regions,
            &mut diagnostics,
        )?;
    }

    Ok(Worksheet {
        version: SUPPORTED_VERSION.to_owned(),
        source: SourceDocument::from_bytes(bytes),
        metadata,
        regions,
        diagnostics,
    })
}

fn parse_metadata(node: &Node) -> Result<WorksheetMetadata, WorksheetError> {
    let mut metadata = WorksheetMetadata::default();
    for child in node.element_children() {
        match child.name.local_name.as_str() {
            "generator" if child.name.namespace_uri.as_deref() == Some(WS_NS) => {
                metadata.generator = Some(child.text());
            }
            "identityInfo" if child.name.namespace_uri.as_deref() == Some(WS_NS) => {
                metadata.identity_info = parse_identity_info(child);
            }
            "userData" if child.name.namespace_uri.as_deref() == Some(WS_NS) => {
                metadata.user_data = parse_user_data(child)?;
            }
            _ => metadata.opaque_fragments.push(child.opaque()),
        }
    }
    Ok(metadata)
}

fn parse_identity_info(node: &Node) -> WorksheetIdentityInfo {
    let value = |name| node.child(WS_NS, name).map(Node::text);
    WorksheetIdentityInfo {
        document_id: value("documentID"),
        branch_id: value("branchID"),
        version_id: value("versionID"),
        parent_version_id: value("parentVersionID"),
        revision: value("revision"),
        saved_on: value("savedOn"),
        comment: node.child(WS_NS, "comment").map(Node::opaque),
        opaque_fragments: node
            .element_children()
            .filter(|child| {
                child.name.namespace_uri.as_deref() != Some(WS_NS)
                    || !matches!(
                        child.name.local_name.as_str(),
                        "documentID"
                            | "branchID"
                            | "versionID"
                            | "parentVersionID"
                            | "revision"
                            | "savedOn"
                            | "comment"
                    )
            })
            .map(Node::opaque)
            .collect(),
    }
}

fn parse_user_data(node: &Node) -> Result<WorksheetUserData, WorksheetError> {
    let value = |name| node.child(WS_NS, name).map(Node::text);
    let mut custom_values = Vec::new();
    let mut opaque_fragments = Vec::new();
    if let Some(container) = node.child(WS_NS, "customValues") {
        for child in container.element_children() {
            if child.is(WS_NS, "customValue") {
                custom_values.push(parse_custom_value(child)?);
            } else {
                opaque_fragments.push(child.opaque());
            }
        }
    }
    opaque_fragments.extend(
        node.element_children()
            .filter(|child| {
                child.name.namespace_uri.as_deref() != Some(WS_NS)
                    || !matches!(
                        child.name.local_name.as_str(),
                        "author"
                            | "company"
                            | "description"
                            | "keywords"
                            | "revisedBy"
                            | "title"
                            | "customValues"
                    )
            })
            .map(Node::opaque),
    );
    Ok(WorksheetUserData {
        author: value("author"),
        company: value("company"),
        description: value("description"),
        keywords: value("keywords"),
        revised_by: value("revisedBy"),
        title: value("title"),
        custom_values,
        opaque_fragments,
    })
}

fn parse_custom_value(node: &Node) -> Result<WorksheetCustomValue, WorksheetError> {
    let name = node
        .attribute("name")
        .ok_or(WorksheetError::MalformedCustomValue)?
        .to_owned();
    let kind = match node
        .attribute("type")
        .ok_or(WorksheetError::MalformedCustomValue)?
    {
        "date" => CustomValueKind::Date,
        "number" => CustomValueKind::Number,
        "text" => CustomValueKind::Text,
        "yesno" => CustomValueKind::YesNo,
        _ => return Err(WorksheetError::MalformedCustomValue),
    };
    Ok(WorksheetCustomValue {
        name,
        kind,
        value: node.text(),
        span: node.span,
    })
}

fn discover_regions(
    node: &Node,
    limits: WorksheetLimits,
    identifiers: &mut HashSet<u64>,
    regions: &mut Vec<Region>,
    diagnostics: &mut Vec<Diagnostic>,
) -> Result<(), WorksheetError> {
    for child in node.element_children() {
        if child.is(WS_NS, "region") {
            if regions.len() >= limits.max_regions {
                return Err(WorksheetError::LimitExceeded(WorksheetLimit::Regions));
            }
            let region = parse_region(child, regions.len(), limits, diagnostics)?;
            if !identifiers.insert(region.id) {
                return Err(WorksheetError::DuplicateRegionId);
            }
            regions.push(region);
            for descendant in child
                .element_children()
                .filter(|node| node.is(WS_NS, "area"))
            {
                discover_regions(descendant, limits, identifiers, regions, diagnostics)?;
            }
        } else if child.is(WS_NS, "area") || child.is(WS_NS, "regions") {
            discover_regions(child, limits, identifiers, regions, diagnostics)?;
        }
    }
    Ok(())
}

fn parse_region(
    node: &Node,
    source_ordinal: usize,
    limits: WorksheetLimits,
    diagnostics: &mut Vec<Diagnostic>,
) -> Result<Region, WorksheetError> {
    let id = node
        .attribute("region-id")
        .ok_or(WorksheetError::MissingRegionId)?
        .parse::<u64>()
        .map_err(|_| WorksheetError::MalformedRegionId)?;
    let layout = RegionLayout {
        top: coordinate(node, "top")?,
        left: coordinate(node, "left")?,
        height: coordinate(node, "height")?,
        width: coordinate(node, "width")?,
        z_order: node
            .attribute("z-order")
            .map(str::parse::<i64>)
            .transpose()
            .map_err(|_| WorksheetError::MalformedZOrder)?
            .unwrap_or(0),
    };
    let content_node = node
        .element_children()
        .next()
        .ok_or(WorksheetError::MalformedXml)?;
    let content = if content_node.is(WS_NS, "text") {
        RegionContent::Text(parse_text(content_node, diagnostics)?)
    } else if content_node.is(WS_NS, "math") {
        RegionContent::Math(parse_math(
            content_node,
            limits.max_ast_nodes,
            limits.max_matrix_elements,
            diagnostics,
        )?)
    } else if content_node.is(WS_NS, "plot") {
        RegionContent::Plot(parse_plot(content_node)?)
    } else if content_node.is(WS_NS, "picture") {
        RegionContent::Picture(parse_picture(content_node)?)
    } else if content_node.is(WS_NS, "area") {
        RegionContent::Area(content_node.opaque())
    } else {
        diagnostics.push(Diagnostic::warning(
            DiagnosticCode::UnknownRegionContent,
            Some(source_ordinal),
        ));
        RegionContent::Opaque(content_node.opaque())
    };
    Ok(Region {
        id,
        source_ordinal,
        span: node.span,
        layout,
        content,
    })
}

fn coordinate(node: &Node, field: &'static str) -> Result<SourceNumber, WorksheetError> {
    let lexeme = node
        .attribute(field)
        .ok_or(WorksheetError::InvalidCoordinate {
            field,
            reason: CoordinateError::Missing,
        })?;
    let value = lexeme
        .parse::<f64>()
        .map_err(|_| WorksheetError::InvalidCoordinate {
            field,
            reason: CoordinateError::Malformed,
        })?;
    if !value.is_finite() {
        return Err(WorksheetError::InvalidCoordinate {
            field,
            reason: CoordinateError::NonFinite,
        });
    }
    Ok(SourceNumber {
        value,
        lexeme: lexeme.to_owned(),
    })
}

fn parse_text(
    node: &Node,
    diagnostics: &mut Vec<Diagnostic>,
) -> Result<TextRegion, WorksheetError> {
    let paragraphs = node
        .element_children()
        .filter(|child| child.is(WS_NS, "p"))
        .map(|paragraph| {
            Ok(TextParagraph {
                style: TextValue(
                    paragraph
                        .attribute("style")
                        .ok_or(WorksheetError::MissingTextStyle)?
                        .to_owned(),
                ),
                runs: parse_runs(paragraph, diagnostics),
                span: paragraph.span,
            })
        })
        .collect::<Result<Vec<_>, WorksheetError>>()?;
    Ok(TextRegion {
        paragraphs,
        span: node.span,
    })
}

fn parse_runs(node: &Node, diagnostics: &mut Vec<Diagnostic>) -> Vec<TextRun> {
    node.children
        .iter()
        .map(|child| match child {
            Child::Text { value, span } => TextRun::Text {
                value: TextValue(value.clone()),
                span: *span,
            },
            Child::Node(child) => match inline_kind(child) {
                Some(kind) => TextRun::Inline {
                    kind,
                    attributes: child
                        .attributes
                        .iter()
                        .map(|attribute| InlineAttribute {
                            name: attribute.name.clone(),
                            value: TextValue(attribute.value.clone()),
                        })
                        .collect(),
                    children: parse_runs(child, diagnostics),
                    span: child.span,
                },
                None => {
                    diagnostics.push(Diagnostic::warning(DiagnosticCode::UnknownInlineNode, None));
                    TextRun::Opaque(child.opaque())
                }
            },
        })
        .collect()
}

fn inline_kind(node: &Node) -> Option<InlineKind> {
    if node.name.namespace_uri.as_deref() != Some(WS_NS) {
        return None;
    }
    match node.name.local_name.as_str() {
        "b" => Some(InlineKind::Bold),
        "i" => Some(InlineKind::Italic),
        "u" => Some(InlineKind::Underline),
        "so" => Some(InlineKind::StrikeOut),
        "sub" => Some(InlineKind::Subscript),
        "sup" => Some(InlineKind::Superscript),
        "c" => Some(InlineKind::Color),
        "f" => Some(InlineKind::Font),
        "inlineAttr" => Some(InlineKind::InlineAttribute),
        "link" => Some(InlineKind::Link),
        "br" => Some(InlineKind::Break),
        "tab" => Some(InlineKind::Tab),
        "sp" => Some(InlineKind::Space),
        _ => None,
    }
}

fn parse_math(
    node: &Node,
    max_ast_nodes: usize,
    max_matrix_elements: usize,
    diagnostics: &mut Vec<Diagnostic>,
) -> Result<MathRegion, WorksheetError> {
    let result_format = node
        .child(WS_NS, "resultFormat")
        .map(parse_result_format)
        .transpose()?;
    let candidates: Vec<_> = node
        .element_children()
        .filter(|child| !child.is(WS_NS, "resultFormat"))
        .collect();
    if candidates.len() != 1 {
        return Err(WorksheetError::InvalidMathExpressionCount);
    }
    let expression = candidates[0];
    if expression.name.namespace_uri.as_deref() != Some(MATH_NS) {
        return Err(WorksheetError::UnsupportedMathNamespace);
    }
    let outcome = match crate::math_xml::parse_math_expression(
        expression,
        max_ast_nodes,
        max_matrix_elements,
    ) {
        crate::math_xml::MathXmlOutcome::Parsed(expression) => MathParseOutcome::Parsed(expression),
        crate::math_xml::MathXmlOutcome::Invalid(error) => MathParseOutcome::Invalid(error),
        crate::math_xml::MathXmlOutcome::Unsupported => {
            let diagnostic = Diagnostic::warning(DiagnosticCode::UnsupportedMathNode, None);
            diagnostics.push(diagnostic);
            MathParseOutcome::Unsupported(diagnostic)
        }
    };
    Ok(MathRegion {
        disable_calc: boolean_attribute(node, "disable-calc")?,
        optimize: boolean_attribute(node, "optimize")?,
        span: node.span,
        expression_span: expression.span,
        result_format,
        outcome,
    })
}

fn parse_result_format(node: &Node) -> Result<ResultFormat, WorksheetError> {
    let table = node
        .child(WS_NS, "table")
        .map(|table| {
            Ok(OpaqueTableResult {
                span: table.span,
                item_idref: table
                    .attribute("item-idref")
                    .ok_or(WorksheetError::MalformedResultFormat)?
                    .to_owned(),
            })
        })
        .transpose()?;
    Ok(ResultFormat {
        span: node.span,
        table,
    })
}

fn parse_plot(node: &Node) -> Result<PlotRegion, WorksheetError> {
    Ok(PlotRegion {
        item_idref: node.attribute("item-idref").map(str::to_owned),
        disable_calc: boolean_attribute(node, "disable-calc")?,
        span: node.span,
    })
}

fn parse_picture(node: &Node) -> Result<PictureRegion, WorksheetError> {
    let payload = node.element_children().find(|child| {
        child.name.namespace_uri.as_deref() == Some(WS_NS)
            && matches!(child.name.local_name.as_str(), "png" | "jpg" | "metafile")
    });
    let metadata = payload.ok_or(WorksheetError::MalformedPicture)?;
    let kind_name = metadata.name.local_name.as_str();
    let kind = match kind_name.to_ascii_lowercase().as_str() {
        "png" => PictureKind::Png,
        "jpg" | "jpeg" => PictureKind::Jpg,
        "metafile" => PictureKind::Metafile,
        _ => return Err(WorksheetError::MalformedPicture),
    };
    Ok(PictureRegion {
        kind,
        item_idref: metadata
            .attribute("item-idref")
            .ok_or(WorksheetError::MalformedPicture)?
            .to_owned(),
        display_width: optional_finite_number(metadata, "display-width")?,
        display_height: optional_finite_number(metadata, "display-height")?,
        x_extent: optional_finite_number(metadata, "x-extent")?,
        y_extent: optional_finite_number(metadata, "y-extent")?,
        quality: picture_quality(metadata, kind)?,
        mapping_mode: metadata
            .attribute("mapping-mode")
            .map(str::parse::<i64>)
            .transpose()
            .map_err(|_| WorksheetError::MalformedPicture)?,
        span: node.span,
    })
    .and_then(validate_picture)
}

fn validate_picture(picture: PictureRegion) -> Result<PictureRegion, WorksheetError> {
    if picture.kind == PictureKind::Metafile
        && (picture.x_extent.is_none()
            || picture.y_extent.is_none()
            || picture.mapping_mode.is_none())
    {
        return Err(WorksheetError::MalformedPicture);
    }
    Ok(picture)
}

fn picture_quality(node: &Node, kind: PictureKind) -> Result<Option<u8>, WorksheetError> {
    let quality = node
        .attribute("quality")
        .map(str::parse::<u8>)
        .transpose()
        .map_err(|_| WorksheetError::MalformedPicture)?;
    if quality.is_some_and(|value| !(1..=100).contains(&value)) {
        return Err(WorksheetError::MalformedPicture);
    }
    Ok(quality.or_else(|| (kind == PictureKind::Jpg).then_some(75)))
}

fn optional_finite_number(
    node: &Node,
    attribute: &str,
) -> Result<Option<SourceNumber>, WorksheetError> {
    node.attribute(attribute)
        .map(|lexeme| {
            let value = lexeme
                .parse::<f64>()
                .map_err(|_| WorksheetError::MalformedPicture)?;
            if !value.is_finite() {
                return Err(WorksheetError::MalformedPicture);
            }
            Ok(SourceNumber {
                value,
                lexeme: lexeme.to_owned(),
            })
        })
        .transpose()
}

fn boolean_attribute(node: &Node, name: &str) -> Result<bool, WorksheetError> {
    match node.attribute(name) {
        None | Some("false" | "0") => Ok(false),
        Some("true" | "1") => Ok(true),
        Some(_) => Err(WorksheetError::MalformedBoolean),
    }
}
