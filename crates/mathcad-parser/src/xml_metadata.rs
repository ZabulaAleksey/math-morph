use std::str;

use quick_xml::events::{BytesStart, Event};
use quick_xml::name::{PrefixDeclaration, ResolveResult};
use quick_xml::{NsReader, XmlVersion};
use thiserror::Error;

const MIB: usize = 1024 * 1024;
const XSI_NAMESPACE: &str = "http://www.w3.org/2001/XMLSchema-instance";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct XmlMetadataLimits {
    pub max_input_bytes: usize,
    pub max_namespace_declarations: usize,
    pub max_root_attributes: usize,
    pub max_attribute_value_bytes: usize,
}

impl Default for XmlMetadataLimits {
    fn default() -> Self {
        Self {
            max_input_bytes: 32 * MIB,
            max_namespace_declarations: 64,
            max_root_attributes: 256,
            max_attribute_value_bytes: 16 * 1024,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NamespaceBinding {
    pub prefix: Option<String>,
    pub namespace_uri: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SchemaLocation {
    pub namespace_uri: Option<String>,
    pub location: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct XmlMetadata {
    pub root_local_name: String,
    pub root_namespace_uri: Option<String>,
    pub namespace_bindings: Vec<NamespaceBinding>,
    pub schema_locations: Vec<SchemaLocation>,
    pub declared_encoding: Option<String>,
}

#[derive(Debug, Error)]
pub enum XmlMetadataError {
    #[error("XML input exceeds its configured limit")]
    InputLimitExceeded,
    #[error("XML DOCTYPE is forbidden")]
    DoctypeForbidden,
    #[error("XML uses an unsupported encoding")]
    UnsupportedEncoding,
    #[error("XML root element is missing")]
    MissingRoot,
    #[error("XML uses an undeclared namespace prefix")]
    UnknownNamespacePrefix,
    #[error("XML root attributes exceed their configured limit")]
    AttributeLimitExceeded,
    #[error("XML namespace declarations exceed their configured limit")]
    NamespaceLimitExceeded,
    #[error("XML schema location is malformed")]
    MalformedSchemaLocation,
    #[error("XML metadata is malformed")]
    MalformedXml,
}

pub fn inspect_xml_metadata(
    bytes: &[u8],
    limits: XmlMetadataLimits,
) -> Result<XmlMetadata, XmlMetadataError> {
    if bytes.len() > limits.max_input_bytes {
        return Err(XmlMetadataError::InputLimitExceeded);
    }
    if starts_with_utf16_bom(bytes) {
        return Err(XmlMetadataError::UnsupportedEncoding);
    }

    let mut reader = NsReader::from_reader(bytes);
    reader
        .resolver_mut()
        .set_max_declarations_per_element(limits.max_namespace_declarations);
    let mut buffer = Vec::new();
    let mut declared_encoding = None;

    loop {
        let event = reader.read_event_into(&mut buffer).map_err(map_xml_error)?;
        match event {
            Event::Decl(declaration) => {
                if let Some(encoding) = declaration.encoding() {
                    let encoding = encoding.map_err(|_| XmlMetadataError::MalformedXml)?;
                    let encoding = str::from_utf8(&encoding)
                        .map_err(|_| XmlMetadataError::UnsupportedEncoding)?
                        .to_owned();
                    if !encoding.eq_ignore_ascii_case("UTF-8") {
                        return Err(XmlMetadataError::UnsupportedEncoding);
                    }
                    declared_encoding = Some(encoding);
                }
            }
            Event::DocType(_) => return Err(XmlMetadataError::DoctypeForbidden),
            Event::Start(root) | Event::Empty(root) => {
                let root_namespace_uri = match reader.resolver().resolve_element(root.name()).0 {
                    ResolveResult::Bound(namespace) => Some(
                        str::from_utf8(namespace.as_ref())
                            .map_err(|_| XmlMetadataError::MalformedXml)?
                            .to_owned(),
                    ),
                    ResolveResult::Unbound => None,
                    ResolveResult::Unknown(_) => {
                        return Err(XmlMetadataError::UnknownNamespacePrefix);
                    }
                };
                return build_metadata(
                    &reader,
                    root_namespace_uri,
                    &root,
                    declared_encoding,
                    limits,
                );
            }
            Event::Text(text) if text.iter().copied().all(|byte| byte.is_ascii_whitespace()) => {}
            Event::Comment(_) | Event::PI(_) => {}
            Event::Eof => return Err(XmlMetadataError::MissingRoot),
            _ => return Err(XmlMetadataError::MalformedXml),
        }
        buffer.clear();
    }
}

fn build_metadata(
    reader: &NsReader<&[u8]>,
    root_namespace_uri: Option<String>,
    root: &BytesStart<'_>,
    declared_encoding: Option<String>,
    limits: XmlMetadataLimits,
) -> Result<XmlMetadata, XmlMetadataError> {
    let root_local_name = str::from_utf8(root.local_name().as_ref())
        .map_err(|_| XmlMetadataError::MalformedXml)?
        .to_owned();
    let mut namespace_bindings = Vec::new();
    for (prefix, namespace) in reader.resolver().bindings_of(1) {
        let prefix = match prefix {
            PrefixDeclaration::Default => None,
            PrefixDeclaration::Named(prefix) => Some(
                str::from_utf8(prefix)
                    .map_err(|_| XmlMetadataError::MalformedXml)?
                    .to_owned(),
            ),
        };
        namespace_bindings.push(NamespaceBinding {
            prefix,
            namespace_uri: str::from_utf8(namespace.as_ref())
                .map_err(|_| XmlMetadataError::MalformedXml)?
                .to_owned(),
        });
    }

    let mut attributes = 0_usize;
    let mut schema_locations = Vec::new();
    for attribute in root.attributes() {
        attributes += 1;
        if attributes > limits.max_root_attributes {
            return Err(XmlMetadataError::AttributeLimitExceeded);
        }
        let attribute = attribute.map_err(|_| XmlMetadataError::MalformedXml)?;
        if attribute.value.len() > limits.max_attribute_value_bytes {
            return Err(XmlMetadataError::AttributeLimitExceeded);
        }
        if attribute.key.as_namespace_binding().is_some() {
            continue;
        }
        let (attribute_namespace, local_name) = reader.resolver().resolve_attribute(attribute.key);
        let is_xsi_namespace = match attribute_namespace {
            ResolveResult::Bound(namespace) => {
                str::from_utf8(namespace.as_ref()).map_err(|_| XmlMetadataError::MalformedXml)?
                    == XSI_NAMESPACE
            }
            ResolveResult::Unbound => false,
            ResolveResult::Unknown(_) => return Err(XmlMetadataError::UnknownNamespacePrefix),
        };
        if !is_xsi_namespace {
            continue;
        }
        let value = attribute
            .decoded_and_normalized_value(XmlVersion::Implicit1_0, reader.decoder())
            .map_err(|_| XmlMetadataError::MalformedXml)?;
        match local_name.as_ref() {
            b"schemaLocation" => {
                let tokens: Vec<&str> = value.split_whitespace().collect();
                if tokens.len() % 2 != 0 {
                    return Err(XmlMetadataError::MalformedSchemaLocation);
                }
                schema_locations.extend(tokens.chunks_exact(2).map(|pair| SchemaLocation {
                    namespace_uri: Some(pair[0].to_owned()),
                    location: pair[1].to_owned(),
                }));
            }
            b"noNamespaceSchemaLocation" => schema_locations.push(SchemaLocation {
                namespace_uri: None,
                location: value.into_owned(),
            }),
            _ => {}
        }
    }

    Ok(XmlMetadata {
        root_local_name,
        root_namespace_uri,
        namespace_bindings,
        schema_locations,
        declared_encoding,
    })
}

fn starts_with_utf16_bom(bytes: &[u8]) -> bool {
    bytes.starts_with(&[0xff, 0xfe]) || bytes.starts_with(&[0xfe, 0xff])
}

fn map_xml_error(error: quick_xml::Error) -> XmlMetadataError {
    if error.to_string().contains("namespace declarations") {
        XmlMetadataError::NamespaceLimitExceeded
    } else {
        XmlMetadataError::MalformedXml
    }
}
