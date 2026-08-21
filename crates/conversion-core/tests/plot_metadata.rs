use conversion_core::{
    ConversionOptions, ConversionPipeline, ConversionRequest, DiagnosticCode, Fidelity,
    PartialPolicy, TargetFormat,
};
use document_ir::BlockContentIr;
use exporter_docx::DocxValidator;

const WS: &str = "http://schemas.mathsoft.com/worksheet30";
const ML: &str = "http://schemas.mathsoft.com/math30";

fn worksheet(plot: &str) -> Vec<u8> {
    format!(
        r#"<?xml version="1.0"?><x:worksheet xmlns:x="{WS}" xmlns:m="{ML}" version="3.0.3"><x:regions><x:region region-id="1" top="0" left="0" height="10" width="20"><x:text><x:p style="Normal">kept</x:p></x:text></x:region><x:region region-id="2" top="1" left="0" height="10" width="20">{plot}</x:region></x:regions></x:worksheet>"#
    )
    .into_bytes()
}

#[test]
fn partial_conversion_preserves_confirmed_plot_metadata_in_ir_v3() {
    let outcome = ConversionPipeline::new()
        .convert(ConversionRequest::new(
            worksheet(r#"<x:plot item-idref="plot-item-2" disable-calc="1"/>"#),
            Some("plot.xmcd".into()),
            TargetFormat::Docx,
            ConversionOptions {
                partial_policy: PartialPolicy::AllowSafePartial,
                ..ConversionOptions::default()
            },
        ))
        .expect("safe partial conversion");

    DocxValidator::default()
        .validate(&outcome.artifact)
        .expect("valid supported projection");
    assert_eq!(outcome.document.schema_version(), 3);
    let v3 = outcome.document.as_v3().expect("plot-aware V3");
    assert_eq!(v3.plot_metadata.len(), 1);
    assert_eq!(
        v3.plot_metadata[0].item_idref.as_deref(),
        Some("plot-item-2")
    );
    assert!(v3.plot_metadata[0].disable_calc);
    assert!(matches!(
        v3.document.pages[0].blocks[1].content,
        BlockContentIr::Plot(_)
    ));
    assert!(
        outcome
            .report
            .items
            .iter()
            .any(|item| item.fidelity == Fidelity::Unsupported)
    );
    assert!(
        outcome
            .report
            .diagnostics
            .iter()
            .any(|item| item.code == DiagnosticCode::UnsupportedRegion)
    );
}

#[test]
fn absent_optional_plot_reference_is_preserved_without_guessing() {
    let outcome = ConversionPipeline::new()
        .convert(ConversionRequest::new(
            worksheet(r#"<x:plot/>"#),
            Some("plot.xmcd".into()),
            TargetFormat::Docx,
            ConversionOptions {
                partial_policy: PartialPolicy::AllowSafePartial,
                ..ConversionOptions::default()
            },
        ))
        .expect("safe partial conversion");
    let metadata = &outcome.document.as_v3().unwrap().plot_metadata[0];
    assert!(metadata.item_idref.is_none());
    assert!(!metadata.disable_calc);
}
