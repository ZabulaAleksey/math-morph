use std::collections::BTreeSet;
use std::fmt;
use std::str;
use std::sync::Arc;

use quick_xml::events::{BytesStart, Event};
use quick_xml::name::ResolveResult;
use quick_xml::{NsReader, XmlVersion};

use crate::{DocxLimit, DocxLimits, DocxValidationError};

#[derive(Clone, Eq, PartialEq)]
pub(crate) struct XmlAttribute {
    pub(crate) namespace: Option<Arc<str>>,
    pub(crate) local: String,
    pub(crate) value: String,
}

impl fmt::Debug for XmlAttribute {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("XmlAttribute")
            .field("has_namespace", &self.namespace.is_some())
            .field("local_bytes", &self.local.len())
            .field("value_bytes", &self.value.len())
            .finish()
    }
}

#[derive(Clone, Eq, PartialEq)]
pub(crate) struct XmlNode {
    pub(crate) namespace: Option<Arc<str>>,
    pub(crate) local: String,
    pub(crate) attributes: Vec<XmlAttribute>,
    pub(crate) children: Vec<XmlNode>,
    pub(crate) text: String,
    pub(crate) source_start: usize,
    pub(crate) source_end: usize,
}

impl fmt::Debug for XmlNode {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("XmlNode")
            .field("has_namespace", &self.namespace.is_some())
            .field("local_bytes", &self.local.len())
            .field("attribute_count", &self.attributes.len())
            .field("child_count", &self.children.len())
            .field("text_bytes", &self.text.len())
            .field(
                "source_bytes",
                &self.source_end.saturating_sub(self.source_start),
            )
            .finish()
    }
}

impl XmlNode {
    pub(crate) fn is(&self, namespace: &str, local: &str) -> bool {
        self.namespace.as_deref() == Some(namespace) && self.local == local
    }

    pub(crate) fn attribute(&self, namespace: Option<&str>, local: &str) -> Option<&str> {
        self.attributes
            .iter()
            .find(|attribute| {
                attribute.namespace.as_deref() == namespace && attribute.local == local
            })
            .map(|attribute| attribute.value.as_str())
    }

    pub(crate) fn descendants<'a>(&'a self, output: &mut Vec<&'a XmlNode>) {
        output.push(self);
        for child in &self.children {
            child.descendants(output);
        }
    }
}

pub(crate) fn parse_xml(bytes: &[u8], limits: &DocxLimits) -> Result<XmlNode, DocxValidationError> {
    if u64::try_from(bytes.len()).unwrap_or(u64::MAX) > limits.max_xml_bytes {
        return Err(DocxValidationError::LimitExceeded(DocxLimit::XmlBytes));
    }
    str::from_utf8(bytes).map_err(|_| DocxValidationError::InvalidDocumentXml)?;
    let mut reader = NsReader::from_reader(bytes);
    let mut buffer = Vec::new();
    let mut stack = Vec::new();
    let mut root = None;
    let mut nodes = 0_usize;
    let mut namespaces = BTreeSet::new();
    loop {
        let event_start = usize::try_from(reader.buffer_position())
            .map_err(|_| DocxValidationError::InvalidDocumentXml)?;
        let (resolution, event) = reader
            .read_resolved_event_into(&mut buffer)
            .map_err(|_| DocxValidationError::InvalidDocumentXml)?;
        let element_namespace = resolve_namespace(resolution, &mut namespaces)?;
        let event_end = usize::try_from(reader.buffer_position())
            .map_err(|_| DocxValidationError::InvalidDocumentXml)?;
        match event {
            Event::Decl(declaration) => {
                if declaration
                    .encoding()
                    .transpose()
                    .map_err(|_| DocxValidationError::InvalidDocumentXml)?
                    .is_some_and(|encoding| !encoding.eq_ignore_ascii_case(b"UTF-8"))
                {
                    return Err(DocxValidationError::InvalidDocumentXml);
                }
            }
            Event::DocType(_) => return Err(DocxValidationError::DtdForbidden),
            Event::Start(start) => {
                if stack.len() >= limits.max_xml_depth {
                    return Err(DocxValidationError::LimitExceeded(DocxLimit::XmlDepth));
                }
                count_node(&mut nodes, limits)?;
                let node = start_node(
                    &reader,
                    element_namespace,
                    &start,
                    &mut namespaces,
                    event_start,
                    event_end,
                )?;
                stack.push(node);
            }
            Event::Empty(start) => {
                if stack.len() >= limits.max_xml_depth {
                    return Err(DocxValidationError::LimitExceeded(DocxLimit::XmlDepth));
                }
                count_node(&mut nodes, limits)?;
                let node = start_node(
                    &reader,
                    element_namespace,
                    &start,
                    &mut namespaces,
                    event_start,
                    event_end,
                )?;
                push_node(&mut stack, &mut root, node)?;
            }
            Event::End(_) => {
                let mut node = stack.pop().ok_or(DocxValidationError::InvalidDocumentXml)?;
                node.source_end = event_end;
                push_node(&mut stack, &mut root, node)?;
            }
            Event::Text(text) => {
                let value = text
                    .xml_content(XmlVersion::Implicit1_0)
                    .map_err(|_| DocxValidationError::InvalidDocumentXml)?;
                push_text(&mut stack, &value)?;
            }
            Event::CData(text) => {
                let value = text
                    .xml_content(XmlVersion::Implicit1_0)
                    .map_err(|_| DocxValidationError::InvalidDocumentXml)?;
                push_text(&mut stack, &value)?;
            }
            Event::GeneralRef(reference) => {
                let name = reference
                    .decode()
                    .map_err(|_| DocxValidationError::InvalidDocumentXml)?;
                if !matches!(name.as_ref(), "amp" | "lt" | "gt" | "quot" | "apos")
                    && !name.starts_with('#')
                {
                    return Err(DocxValidationError::InvalidDocumentXml);
                }
                let escaped = format!("&{name};");
                let value = quick_xml::escape::unescape(&escaped)
                    .map_err(|_| DocxValidationError::InvalidDocumentXml)?;
                push_text(&mut stack, &value)?;
            }
            Event::Comment(_) => {}
            Event::PI(_) => return Err(DocxValidationError::InvalidDocumentXml),
            Event::Eof => break,
        }
        buffer.clear();
    }
    if !stack.is_empty() {
        return Err(DocxValidationError::InvalidDocumentXml);
    }
    root.ok_or(DocxValidationError::InvalidDocumentXml)
}

fn count_node(nodes: &mut usize, limits: &DocxLimits) -> Result<(), DocxValidationError> {
    *nodes = nodes
        .checked_add(1)
        .ok_or(DocxValidationError::LimitExceeded(DocxLimit::XmlNodes))?;
    if *nodes > limits.max_xml_nodes {
        return Err(DocxValidationError::LimitExceeded(DocxLimit::XmlNodes));
    }
    Ok(())
}

fn start_node(
    reader: &NsReader<&[u8]>,
    namespace: Option<Arc<str>>,
    start: &BytesStart<'_>,
    namespaces: &mut BTreeSet<Arc<str>>,
    source_start: usize,
    source_end: usize,
) -> Result<XmlNode, DocxValidationError> {
    let local = str::from_utf8(start.local_name().as_ref())
        .map_err(|_| DocxValidationError::InvalidDocumentXml)?
        .to_owned();
    let mut attributes = Vec::new();
    let mut expanded_attributes = BTreeSet::new();
    for attribute in start.attributes() {
        let attribute = attribute.map_err(|_| DocxValidationError::InvalidDocumentXml)?;
        let value = attribute
            .decoded_and_normalized_value(XmlVersion::Implicit1_0, reader.decoder())
            .map_err(|_| DocxValidationError::InvalidDocumentXml)?
            .into_owned();
        if !value.chars().all(is_xml_10_char) {
            return Err(DocxValidationError::InvalidDocumentXml);
        }
        if attribute.key.as_namespace_binding().is_some() {
            continue;
        }
        let (attribute_namespace, local_name) = reader.resolver().resolve_attribute(attribute.key);
        let namespace = resolve_namespace(attribute_namespace, namespaces)?;
        let local = str::from_utf8(local_name.as_ref())
            .map_err(|_| DocxValidationError::InvalidDocumentXml)?
            .to_owned();
        if !expanded_attributes.insert((namespace.clone(), local.clone())) {
            return Err(DocxValidationError::InvalidDocumentXml);
        }
        attributes.push(XmlAttribute {
            namespace,
            local,
            value,
        });
    }
    Ok(XmlNode {
        namespace,
        local,
        attributes,
        children: Vec::new(),
        text: String::new(),
        source_start,
        source_end,
    })
}

fn push_text(stack: &mut [XmlNode], value: &str) -> Result<(), DocxValidationError> {
    if !value.chars().all(is_xml_10_char) {
        return Err(DocxValidationError::InvalidDocumentXml);
    }
    if let Some(parent) = stack.last_mut() {
        parent.text.push_str(value);
        Ok(())
    } else if value.trim().is_empty() {
        Ok(())
    } else {
        Err(DocxValidationError::InvalidDocumentXml)
    }
}

fn resolve_namespace(
    resolution: ResolveResult<'_>,
    namespaces: &mut BTreeSet<Arc<str>>,
) -> Result<Option<Arc<str>>, DocxValidationError> {
    match resolution {
        ResolveResult::Bound(namespace) => {
            let value = str::from_utf8(namespace.as_ref())
                .map_err(|_| DocxValidationError::InvalidDocumentXml)?;
            if let Some(existing) = namespaces.get(value) {
                return Ok(Some(Arc::clone(existing)));
            }
            let value: Arc<str> = Arc::from(value);
            namespaces.insert(Arc::clone(&value));
            Ok(Some(value))
        }
        ResolveResult::Unbound => Ok(None),
        ResolveResult::Unknown(_) => Err(DocxValidationError::InvalidDocumentXml),
    }
}

fn push_node(
    stack: &mut [XmlNode],
    root: &mut Option<XmlNode>,
    node: XmlNode,
) -> Result<(), DocxValidationError> {
    if let Some(parent) = stack.last_mut() {
        parent.children.push(node);
    } else if root.replace(node).is_some() {
        return Err(DocxValidationError::InvalidDocumentXml);
    }
    Ok(())
}

pub(crate) fn is_xml_10_char(character: char) -> bool {
    matches!(character, '\u{9}' | '\u{A}' | '\u{D}')
        || ('\u{20}'..='\u{D7FF}').contains(&character)
        || ('\u{E000}'..='\u{FFFD}').contains(&character)
        || ('\u{10000}'..='\u{10FFFF}').contains(&character)
}

pub(crate) fn escape_text(value: &str, output: &mut String) -> Result<(), crate::DocxError> {
    if !value.chars().all(is_xml_10_char) {
        return Err(crate::DocxError::InvalidXmlText);
    }
    for character in value.chars() {
        match character {
            '&' => output.push_str("&amp;"),
            '<' => output.push_str("&lt;"),
            '>' => output.push_str("&gt;"),
            _ => output.push(character),
        }
    }
    Ok(())
}

pub(crate) fn escape_attribute(value: &str, output: &mut String) -> Result<(), crate::DocxError> {
    if !value.chars().all(is_xml_10_char) {
        return Err(crate::DocxError::InvalidXmlText);
    }
    for character in value.chars() {
        match character {
            '&' => output.push_str("&amp;"),
            '<' => output.push_str("&lt;"),
            '>' => output.push_str("&gt;"),
            '"' => output.push_str("&quot;"),
            '\'' => output.push_str("&apos;"),
            _ => output.push(character),
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use super::parse_xml;
    use crate::{DocxLimits, DocxValidationError};

    #[test]
    fn namespace_uris_are_interned_and_namespace_values_are_decoded() {
        let xml = br#"<root xmlns:x="urn:shared"><x:item/><x:item/></root>"#;
        let root = parse_xml(xml, &DocxLimits::default()).expect("valid XML");
        let first = root.children[0].namespace.as_ref().expect("namespace");
        let second = root.children[1].namespace.as_ref().expect("namespace");
        assert!(Arc::ptr_eq(first, second));

        let malformed = br#"<root xmlns:x="&undefined;"/>"#;
        assert_eq!(
            parse_xml(malformed, &DocxLimits::default()),
            Err(DocxValidationError::InvalidDocumentXml)
        );

        let invalid_character = br#"<root value="&#1;">&#1;</root>"#;
        assert_eq!(
            parse_xml(invalid_character, &DocxLimits::default()),
            Err(DocxValidationError::InvalidDocumentXml)
        );
    }
}
