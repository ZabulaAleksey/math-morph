use std::cell::Cell;
use std::collections::BTreeSet;
use std::io::{self, Cursor, Seek, SeekFrom, Write};
use std::rc::Rc;

use document_ir::{
    BlockContentIr, DocumentIrV1, FormulaDisplayModeIr, FormulaIr, MediaTypeIr, PageOrientationIr,
    TextBlockIr, TextRunIr, TextStyleIr, VersionedDocumentIr, VerticalAlignIr,
    ports::{AssetResolveError, AssetResolver, EquationExporter},
};
use zip::write::SimpleFileOptions;
use zip::{CompressionMethod, DateTime, System, ZipWriter};

use crate::image::validate_image;
use crate::xml::{escape_attribute, escape_text};
use crate::{DocxError, DocxLimit, DocxLimits, DocxValidator, OmmlLimits, WordEquationExporter};

const ROOT_RELS: &str = "<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?><Relationships xmlns=\"http://schemas.openxmlformats.org/package/2006/relationships\"><Relationship Id=\"rId1\" Type=\"http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument\" Target=\"word/document.xml\"/></Relationships>";

struct PackagePart {
    name: String,
    bytes: Vec<u8>,
}

struct PreparedImage {
    bytes: Vec<u8>,
    media_type: MediaTypeIr,
    extension: &'static str,
    part_name: String,
    relationship_id: String,
    drawing_id: u32,
    width_emu: u64,
    height_emu: u64,
    alt_text: Option<String>,
}

struct LimitedCursor {
    inner: Cursor<Vec<u8>>,
    maximum: u64,
    exceeded: Rc<Cell<bool>>,
}

impl LimitedCursor {
    fn new(maximum: u64) -> (Self, Rc<Cell<bool>>) {
        let exceeded = Rc::new(Cell::new(false));
        (
            Self {
                inner: Cursor::new(Vec::new()),
                maximum,
                exceeded: Rc::clone(&exceeded),
            },
            exceeded,
        )
    }

    fn into_inner(self) -> Vec<u8> {
        self.inner.into_inner()
    }

    fn limit_error(&self) -> io::Error {
        self.exceeded.set(true);
        io::Error::other("DOCX output limit exceeded")
    }
}

impl Write for LimitedCursor {
    fn write(&mut self, buffer: &[u8]) -> io::Result<usize> {
        let end = self
            .inner
            .position()
            .checked_add(u64::try_from(buffer.len()).unwrap_or(u64::MAX))
            .ok_or_else(|| self.limit_error())?;
        if end > self.maximum {
            return Err(self.limit_error());
        }
        self.inner.write(buffer)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.inner.flush()
    }
}

impl Seek for LimitedCursor {
    fn seek(&mut self, position: SeekFrom) -> io::Result<u64> {
        let previous = self.inner.position();
        let result = self.inner.seek(position)?;
        if result > self.maximum {
            self.inner.set_position(previous);
            return Err(self.limit_error());
        }
        Ok(result)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DocxExporter {
    limits: DocxLimits,
}

impl DocxExporter {
    pub const fn new(limits: DocxLimits) -> Self {
        Self { limits }
    }

    pub const fn limits(&self) -> &DocxLimits {
        &self.limits
    }

    pub fn export<R: AssetResolver + ?Sized>(
        &self,
        document: &DocumentIrV1,
        resolver: &R,
    ) -> Result<Vec<u8>, DocxError> {
        document.validate()?;
        if document.pages.len() != 1 {
            return Err(DocxError::MultiplePagesUnsupported);
        }
        if document.pages[0].blocks.iter().any(|block| {
            !matches!(
                block.content,
                BlockContentIr::Text(_) | BlockContentIr::Image(_) | BlockContentIr::Equation(_)
            )
        }) {
            return Err(DocxError::UnsupportedContent);
        }
        let images = self.prepare_images(document, resolver)?;
        let content_types = self.render_content_types(&images)?;
        let document_xml = self.render_document(document, &images)?;
        let document_relationships = if images.is_empty() {
            None
        } else {
            Some(self.render_document_relationships(&images)?)
        };
        let mut xml_bytes = u64::try_from(content_types.len())
            .unwrap_or(u64::MAX)
            .checked_add(u64::try_from(ROOT_RELS.len()).unwrap_or(u64::MAX))
            .and_then(|value| {
                value.checked_add(u64::try_from(document_xml.len()).unwrap_or(u64::MAX))
            })
            .ok_or(DocxError::LimitExceeded(DocxLimit::XmlBytes))?;
        if let Some(relationships) = &document_relationships {
            xml_bytes = xml_bytes
                .checked_add(u64::try_from(relationships.len()).unwrap_or(u64::MAX))
                .ok_or(DocxError::LimitExceeded(DocxLimit::XmlBytes))?;
        }
        if xml_bytes > self.limits.max_xml_bytes {
            return Err(DocxError::LimitExceeded(DocxLimit::XmlBytes));
        }
        let mut parts = vec![
            PackagePart {
                name: "[Content_Types].xml".to_owned(),
                bytes: content_types.into_bytes(),
            },
            PackagePart {
                name: "_rels/.rels".to_owned(),
                bytes: ROOT_RELS.as_bytes().to_vec(),
            },
            PackagePart {
                name: "word/document.xml".to_owned(),
                bytes: document_xml.into_bytes(),
            },
        ];
        if let Some(relationships) = document_relationships {
            parts.push(PackagePart {
                name: "word/_rels/document.xml.rels".to_owned(),
                bytes: relationships.into_bytes(),
            });
        }
        parts.extend(images.into_iter().map(|image| PackagePart {
            name: format!("word/{}", image.part_name),
            bytes: image.bytes,
        }));
        self.build_package(&parts)
    }

    pub fn export_versioned<R: AssetResolver + ?Sized>(
        &self,
        document: &VersionedDocumentIr,
        resolver: &R,
    ) -> Result<Vec<u8>, DocxError> {
        self.export(document.as_v1(), resolver)
    }

    fn render_document(
        &self,
        document: &DocumentIrV1,
        images: &[PreparedImage],
    ) -> Result<String, DocxError> {
        let page = &document.pages[0];
        let mut output = String::with_capacity(1024);
        output.push_str("<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?><w:document xmlns:w=\"http://schemas.openxmlformats.org/wordprocessingml/2006/main\"");
        if !images.is_empty() {
            output.push_str(" xmlns:r=\"http://schemas.openxmlformats.org/officeDocument/2006/relationships\" xmlns:wp=\"http://schemas.openxmlformats.org/drawingml/2006/wordprocessingDrawing\" xmlns:a=\"http://schemas.openxmlformats.org/drawingml/2006/main\" xmlns:pic=\"http://schemas.openxmlformats.org/drawingml/2006/picture\"");
        }
        output.push_str("><w:body>");
        self.check_xml(&output)?;
        let mut blocks = 0_usize;
        let mut paragraphs = 0_usize;
        let mut runs = 0_usize;
        let mut image_index = 0_usize;
        for block in &page.blocks {
            increment(&mut blocks, self.limits.max_blocks, DocxLimit::Blocks)?;
            match &block.content {
                BlockContentIr::Text(text) => {
                    self.render_text(text, &mut output, &mut paragraphs, &mut runs)?;
                }
                BlockContentIr::Image(_) => {
                    let image = images
                        .get(image_index)
                        .ok_or(DocxError::GeneratedPackageInvalid)?;
                    image_index = image_index
                        .checked_add(1)
                        .ok_or(DocxError::LimitExceeded(DocxLimit::Images))?;
                    self.render_image(image, &mut output)?;
                }
                BlockContentIr::Equation(formula) => {
                    increment(
                        &mut paragraphs,
                        self.limits.max_paragraphs,
                        DocxLimit::Paragraphs,
                    )?;
                    self.render_equation(formula, &mut output)?;
                }
                _ => return Err(DocxError::UnsupportedContent),
            }
            self.check_xml(&output)?;
        }
        let width = um_to_twips(page.size.width_um)?;
        let height = um_to_twips(page.size.height_um)?;
        let top = um_to_twips(page.margins.top_um)?;
        let right = um_to_twips(page.margins.right_um)?;
        let bottom = um_to_twips(page.margins.bottom_um)?;
        let left = um_to_twips(page.margins.left_um)?;
        output.push_str("<w:sectPr><w:pgSz w:w=\"");
        output.push_str(&width.to_string());
        output.push_str("\" w:h=\"");
        output.push_str(&height.to_string());
        if page.orientation == PageOrientationIr::Landscape {
            output.push_str("\" w:orient=\"landscape");
        }
        output.push_str("\"/><w:pgMar w:top=\"");
        output.push_str(&top.to_string());
        output.push_str("\" w:right=\"");
        output.push_str(&right.to_string());
        output.push_str("\" w:bottom=\"");
        output.push_str(&bottom.to_string());
        output.push_str("\" w:left=\"");
        output.push_str(&left.to_string());
        output.push_str("\"/></w:sectPr></w:body></w:document>");
        self.check_xml(&output)?;
        Ok(output)
    }

    fn render_text(
        &self,
        text: &TextBlockIr,
        output: &mut String,
        paragraphs: &mut usize,
        runs: &mut usize,
    ) -> Result<(), DocxError> {
        for paragraph in &text.paragraphs {
            increment(
                paragraphs,
                self.limits.max_paragraphs,
                DocxLimit::Paragraphs,
            )?;
            output.push_str("<w:p>");
            for run in &paragraph.runs {
                increment(runs, self.limits.max_runs, DocxLimit::Runs)?;
                self.render_run(run, output)?;
                self.check_xml(output)?;
            }
            output.push_str("</w:p>");
        }
        Ok(())
    }

    fn render_run(&self, run: &TextRunIr, output: &mut String) -> Result<(), DocxError> {
        output.push_str("<w:r>");
        render_style(&run.style, output)?;
        output.push_str("<w:t");
        if needs_preserved_space(&run.text) {
            output.push_str(" xml:space=\"preserve\"");
        }
        if run.text.is_empty() {
            output.push_str("/>");
        } else {
            output.push('>');
            escape_text(&run.text, output)?;
            output.push_str("</w:t>");
        }
        output.push_str("</w:r>");
        Ok(())
    }

    fn render_equation(&self, formula: &FormulaIr, output: &mut String) -> Result<(), DocxError> {
        let exporter = WordEquationExporter::new(OmmlLimits {
            max_depth: self.limits.max_equation_depth,
            max_nodes: self.limits.max_equation_nodes,
            max_output_bytes: self.limits.max_equation_output_bytes,
        });
        let fragment = exporter.export(&formula.display)?;
        output.push_str("<w:p>");
        if formula.mode == FormulaDisplayModeIr::Display {
            output.push_str("<m:oMathPara xmlns:m=\"http://schemas.openxmlformats.org/officeDocument/2006/math\">");
        }
        output.push_str(fragment.as_str());
        if formula.mode == FormulaDisplayModeIr::Display {
            output.push_str("</m:oMathPara>");
        }
        output.push_str("</w:p>");
        self.check_xml(output)
    }

    fn prepare_images<R: AssetResolver + ?Sized>(
        &self,
        document: &DocumentIrV1,
        resolver: &R,
    ) -> Result<Vec<PreparedImage>, DocxError> {
        let mut images = Vec::new();
        let mut asset_ids = BTreeSet::new();
        let mut total_asset_bytes = 0_u64;
        for block in &document.pages[0].blocks {
            let BlockContentIr::Image(image) = &block.content else {
                continue;
            };
            if images.len() >= self.limits.max_images {
                return Err(DocxError::LimitExceeded(DocxLimit::Images));
            }
            if !asset_ids.insert(image.asset.id.0.clone()) {
                return Err(DocxError::DuplicateAssetId);
            }
            let size = image.size.ok_or(DocxError::MissingImageSize)?;
            let resolved = resolver
                .resolve(&image.asset)
                .map_err(|error| match error {
                    AssetResolveError::Unavailable => DocxError::MissingAsset,
                    AssetResolveError::Rejected => DocxError::RejectedAsset,
                })?;
            if resolved.media_type != image.asset.media_type {
                return Err(DocxError::MediaTypeMismatch);
            }
            total_asset_bytes = total_asset_bytes
                .checked_add(u64::try_from(resolved.bytes.len()).unwrap_or(u64::MAX))
                .ok_or(DocxError::LimitExceeded(DocxLimit::TotalAssetBytes))?;
            if total_asset_bytes > self.limits.max_total_asset_bytes {
                return Err(DocxError::LimitExceeded(DocxLimit::TotalAssetBytes));
            }
            let info = validate_image(&resolved.bytes, resolved.media_type, &self.limits)?;
            let sequence = images
                .len()
                .checked_add(1)
                .ok_or(DocxError::LimitExceeded(DocxLimit::Images))?;
            let drawing_id = u32::try_from(sequence).map_err(|_| DocxError::ArithmeticOverflow)?;
            images.push(PreparedImage {
                bytes: resolved.bytes,
                media_type: resolved.media_type,
                extension: info.extension,
                part_name: format!("media/image{sequence}.{}", info.extension),
                relationship_id: format!("rId{sequence}"),
                drawing_id,
                width_emu: um_to_emu(size.width_um)?,
                height_emu: um_to_emu(size.height_um)?,
                alt_text: image.alt_text.clone(),
            });
        }
        Ok(images)
    }

    fn render_content_types(&self, images: &[PreparedImage]) -> Result<String, DocxError> {
        let has_png = images
            .iter()
            .any(|image| image.media_type == MediaTypeIr::Png);
        let has_jpeg = images
            .iter()
            .any(|image| image.media_type == MediaTypeIr::Jpeg);
        let mut output = String::with_capacity(768);
        output.push_str("<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?><Types xmlns=\"http://schemas.openxmlformats.org/package/2006/content-types\"><Default Extension=\"rels\" ContentType=\"application/vnd.openxmlformats-package.relationships+xml\"/><Default Extension=\"xml\" ContentType=\"application/xml\"/>");
        if has_png {
            output.push_str("<Default Extension=\"png\" ContentType=\"image/png\"/>");
        }
        if has_jpeg {
            output.push_str("<Default Extension=\"jpg\" ContentType=\"image/jpeg\"/>");
        }
        output.push_str("<Override PartName=\"/word/document.xml\" ContentType=\"application/vnd.openxmlformats-officedocument.wordprocessingml.document.main+xml\"/></Types>");
        self.check_xml(&output)?;
        Ok(output)
    }

    fn render_document_relationships(&self, images: &[PreparedImage]) -> Result<String, DocxError> {
        let mut output = String::with_capacity(512);
        output.push_str("<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?><Relationships xmlns=\"http://schemas.openxmlformats.org/package/2006/relationships\">");
        for image in images {
            output.push_str("<Relationship Id=\"");
            output.push_str(&image.relationship_id);
            output.push_str("\" Type=\"http://schemas.openxmlformats.org/officeDocument/2006/relationships/image\" Target=\"");
            output.push_str(&image.part_name);
            output.push_str("\"/>");
            self.check_xml(&output)?;
        }
        output.push_str("</Relationships>");
        self.check_xml(&output)?;
        Ok(output)
    }

    fn render_image(&self, image: &PreparedImage, output: &mut String) -> Result<(), DocxError> {
        let sequence = image.drawing_id;
        output.push_str("<w:p><w:r><w:drawing><wp:inline distT=\"0\" distB=\"0\" distL=\"0\" distR=\"0\"><wp:extent cx=\"");
        output.push_str(&image.width_emu.to_string());
        output.push_str("\" cy=\"");
        output.push_str(&image.height_emu.to_string());
        output.push_str("\"/><wp:effectExtent l=\"0\" t=\"0\" r=\"0\" b=\"0\"/><wp:docPr id=\"");
        output.push_str(&sequence.to_string());
        output.push_str("\" name=\"Image ");
        output.push_str(&sequence.to_string());
        output.push('"');
        if let Some(alt_text) = &image.alt_text {
            output.push_str(" descr=\"");
            escape_attribute(alt_text, output)?;
            output.push('"');
        }
        output.push_str("><a:extLst/></wp:docPr><wp:cNvGraphicFramePr><a:graphicFrameLocks noChangeAspect=\"1\"/></wp:cNvGraphicFramePr><a:graphic><a:graphicData uri=\"http://schemas.openxmlformats.org/drawingml/2006/picture\"><pic:pic><pic:nvPicPr><pic:cNvPr id=\"0\" name=\"image");
        output.push_str(&sequence.to_string());
        output.push('.');
        output.push_str(image.extension);
        output.push('"');
        if let Some(alt_text) = &image.alt_text {
            output.push_str(" descr=\"");
            escape_attribute(alt_text, output)?;
            output.push('"');
        }
        output.push_str("/><pic:cNvPicPr/></pic:nvPicPr><pic:blipFill><a:blip r:embed=\"");
        output.push_str(&image.relationship_id);
        output.push_str("\"/><a:stretch><a:fillRect/></a:stretch></pic:blipFill><pic:spPr><a:xfrm><a:off x=\"0\" y=\"0\"/><a:ext cx=\"");
        output.push_str(&image.width_emu.to_string());
        output.push_str("\" cy=\"");
        output.push_str(&image.height_emu.to_string());
        output.push_str("\"/></a:xfrm><a:prstGeom prst=\"rect\"><a:avLst/></a:prstGeom></pic:spPr></pic:pic></a:graphicData></a:graphic></wp:inline></w:drawing></w:r></w:p>");
        self.check_xml(output)
    }

    fn build_package(&self, parts: &[PackagePart]) -> Result<Vec<u8>, DocxError> {
        if parts.len() > self.limits.max_entries {
            return Err(DocxError::LimitExceeded(DocxLimit::Entries));
        }
        let uncompressed = parts.iter().try_fold(0_u64, |total, part| {
            total
                .checked_add(u64::try_from(part.bytes.len()).unwrap_or(u64::MAX))
                .ok_or(DocxError::LimitExceeded(DocxLimit::OutputBytes))
        })?;
        if uncompressed > self.limits.max_output_bytes {
            return Err(DocxError::LimitExceeded(DocxLimit::OutputBytes));
        }
        let estimated_stored_size = parts.iter().try_fold(22_u64, |total, part| {
            let name_bytes = u64::try_from(part.name.len()).unwrap_or(u64::MAX);
            total
                .checked_add(u64::try_from(part.bytes.len()).unwrap_or(u64::MAX))
                .and_then(|value| value.checked_add(76))
                .and_then(|value| value.checked_add(name_bytes.checked_mul(2)?))
                .ok_or(DocxError::LimitExceeded(DocxLimit::OutputBytes))
        })?;
        if estimated_stored_size > self.limits.max_output_bytes {
            return Err(DocxError::LimitExceeded(DocxLimit::OutputBytes));
        }
        let (cursor, output_limit_exceeded) = LimitedCursor::new(self.limits.max_output_bytes);
        let mut writer = ZipWriter::new(cursor);
        let options = SimpleFileOptions::default()
            .compression_method(CompressionMethod::Stored)
            .last_modified_time(DateTime::default())
            .system(System::Dos)
            .unix_permissions(0o644);
        for part in parts {
            writer
                .start_file(&part.name, options)
                .map_err(|_| package_write_error(&output_limit_exceeded))?;
            writer
                .write_all(&part.bytes)
                .map_err(|_| package_write_error(&output_limit_exceeded))?;
        }
        let output = writer
            .finish()
            .map_err(|_| package_write_error(&output_limit_exceeded))?
            .into_inner();
        if u64::try_from(output.len()).unwrap_or(u64::MAX) > self.limits.max_output_bytes {
            return Err(DocxError::LimitExceeded(DocxLimit::OutputBytes));
        }
        DocxValidator::new(self.limits)
            .validate(&output)
            .map_err(|_| DocxError::GeneratedPackageInvalid)?;
        Ok(output)
    }

    fn check_xml(&self, value: &str) -> Result<(), DocxError> {
        if u64::try_from(value.len()).unwrap_or(u64::MAX) > self.limits.max_xml_bytes {
            Err(DocxError::LimitExceeded(DocxLimit::XmlBytes))
        } else {
            Ok(())
        }
    }
}

fn package_write_error(output_limit_exceeded: &Cell<bool>) -> DocxError {
    if output_limit_exceeded.get() {
        DocxError::LimitExceeded(DocxLimit::OutputBytes)
    } else {
        DocxError::PackageWrite
    }
}

impl Default for DocxExporter {
    fn default() -> Self {
        Self::new(DocxLimits::default())
    }
}

fn increment(value: &mut usize, maximum: usize, limit: DocxLimit) -> Result<(), DocxError> {
    *value = value
        .checked_add(1)
        .ok_or(DocxError::LimitExceeded(limit))?;
    if *value > maximum {
        Err(DocxError::LimitExceeded(limit))
    } else {
        Ok(())
    }
}

fn render_style(style: &TextStyleIr, output: &mut String) -> Result<(), DocxError> {
    let has_properties = style.bold
        || style.italic
        || style.underline
        || style.strike
        || style.vertical_align != VerticalAlignIr::Baseline
        || style.font_family.is_some()
        || style.font_size_half_points.is_some()
        || style.color.is_some();
    if !has_properties {
        return Ok(());
    }
    output.push_str("<w:rPr>");
    if style.bold {
        output.push_str("<w:b/>");
    }
    if style.italic {
        output.push_str("<w:i/>");
    }
    if style.underline {
        output.push_str("<w:u w:val=\"single\"/>");
    }
    if style.strike {
        output.push_str("<w:strike/>");
    }
    match style.vertical_align {
        VerticalAlignIr::Baseline => {}
        VerticalAlignIr::Subscript => output.push_str("<w:vertAlign w:val=\"subscript\"/>"),
        VerticalAlignIr::Superscript => output.push_str("<w:vertAlign w:val=\"superscript\"/>"),
    }
    if let Some(font) = &style.font_family {
        output.push_str("<w:rFonts w:ascii=\"");
        escape_attribute(font, output)?;
        output.push_str("\" w:hAnsi=\"");
        escape_attribute(font, output)?;
        output.push_str("\"/>");
    }
    if let Some(size) = style.font_size_half_points {
        if size == 0 {
            return Err(DocxError::InvalidTextStyle);
        }
        output.push_str("<w:sz w:val=\"");
        output.push_str(&size.to_string());
        output.push_str("\"/>");
    }
    if let Some(color) = style.color {
        output.push_str("<w:color w:val=\"");
        output.push_str(&format!(
            "{:02X}{:02X}{:02X}",
            color.red, color.green, color.blue
        ));
        output.push_str("\"/>");
    }
    output.push_str("</w:rPr>");
    Ok(())
}

fn needs_preserved_space(value: &str) -> bool {
    value.chars().next().is_some_and(char::is_whitespace)
        || value.chars().next_back().is_some_and(char::is_whitespace)
        || value.contains("  ")
        || value.contains(['\t', '\n', '\r'])
}

fn um_to_twips(value: u64) -> Result<u64, DocxError> {
    let numerator = value
        .checked_mul(1_440)
        .and_then(|value| value.checked_add(12_700))
        .ok_or(DocxError::ArithmeticOverflow)?;
    Ok(numerator / 25_400)
}

fn um_to_emu(value: u64) -> Result<u64, DocxError> {
    value.checked_mul(36).ok_or(DocxError::ArithmeticOverflow)
}
