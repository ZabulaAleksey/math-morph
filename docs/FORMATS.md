# Format Policy

## Inputs

### XMCD
Legacy XML-based Mathcad worksheet family. Parse safely, preserve namespaces/version metadata, region coordinates and unknown nodes as diagnostics/unsupported structures.

### MCDX
Mathcad Prime container family. Treat as untrusted archive/container input. Apply path traversal, entry-count, expanded-size, compression-ratio and nesting limits before parsing contained XML/resources.

## Detection

Never trust extension alone. Record declared extension and detected content format. `FILE_EXTENSION_MISMATCH` may be recoverable if content is confidently recognized and policy allows continuation.

## Outputs

### DOCX — MVP
Text as Word paragraphs/runs, supported equations as editable Office Math/OMML, images/plots with preserved geometry where possible.

### Future
Markdown, PDF, LaTeX, HTML, JSON/web viewer through exporter contracts over `DocumentIR`.

### Charts
Current raster path must coexist with `PlotIR`/future `ChartIR` so future Excel export can create editable charts.

### Diagrams
Current raster path must coexist with `DiagramIR` so future VSDX export can create editable shapes/connectors/groups.

## Unsupported constructs

Unknown/unsupported nodes must never disappear silently. Produce a structured diagnostic and, where safe, partial conversion with an explicit warning/placeholder.
