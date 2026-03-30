# KiCad Extract Diagnostics Inventory

This inventory tracks source-backed diagnostics currently implemented for `ki extract`
parity against KiCad netlist export.

## Implemented Cases

### `missing_input_file`

- KiCad source: `/Users/Daniel/Desktop/kicad/kicad/cli/command_sch_export_netlist.cpp`
- Function: `CLI::SCH_EXPORT_NETLIST_COMMAND::doPerform`
- Trigger: input schematic path does not exist
- KiCad message:
  `Schematic file does not exist or is not accessible`
- Severity: error
- Export behavior: fails before export

### `failed_to_load_schematic`

- KiCad source:
  - `/Users/Daniel/Desktop/kicad/eeschema/eeschema_jobs_handler.cpp`
  - `/Users/Daniel/Desktop/kicad/eeschema/eeschema_helpers.cpp`
- Function: `EESCHEMA_JOBS_HANDLER::getSchematic`
- Trigger: schematic parse/load failure
- KiCad message:
  `Failed to load schematic`
- Severity: error
- Export behavior: fails before export

### `duplicate_references`

- Relevant KiCad source:
  - `/Users/Daniel/Desktop/kicad/eeschema/eeschema_helpers.cpp`
  - `/Users/Daniel/Desktop/kicad/eeschema/eeschema_jobs_handler.cpp`
- Observation:
  a schematic with duplicate explicit references fails load through `kicad-cli` and surfaces the
  generic message `Failed to load schematic`
- Severity: error
- Export behavior: fails before export

### `invalid_property_name`

- Relevant KiCad source:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp`
- Observation:
  a parser-level malformed schematic with an empty property name fails through `kicad-cli`
  but still surfaces only the generic message `Failed to load schematic`
- Severity: error
- Export behavior: fails before export

### `invalid_title_block_property_name`

- Relevant KiCad source:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:2305`
- Internal parser message:
  `Empty property name`
- Observation:
  a title-block property with an empty name fails through `kicad-cli` and surfaces only the
  generic message `Failed to load schematic`
- Severity: error
- Export behavior: fails before export

### `invalid_property_value`

- Relevant KiCad source:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:1099`
- Observation:
  a parser-level malformed schematic with a non-string property value fails through `kicad-cli`
  and surfaces only the generic message `Failed to load schematic`
- Severity: error
- Export behavior: fails before export

### `invalid_text_string`

- Relevant KiCad source:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:1962`
- Observation:
  a schematic text item with a numeric literal instead of a text string fails through `kicad-cli`
  and surfaces only the generic message `Failed to load schematic`
- Severity: error
- Export behavior: fails before export

### `invalid_global_label_shape`

- Relevant KiCad source:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:4687`
- Internal parser branch:
  `Expecting( "input, output, bidirectional, tri_state, passive, dot, round, diamond or rectangle" )`
- Observation:
  a global label with an invalid `shape` token fails through `kicad-cli` and surfaces only the
  generic message `Failed to load schematic`
- Severity: error
- Export behavior: fails before export

### `invalid_global_label_at_arity`

- Relevant KiCad source:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:4648`
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.h:169`
- Internal parser branch:
  `text->SetPosition( parseXY() )`
- Observation:
  a global label with a four-value `(at x y angle extra)` tuple fails through `kicad-cli` and
  surfaces only the generic message `Failed to load schematic`
- Severity: error
- Export behavior: fails before export

### `invalid_hierarchical_label_shape`

- Relevant KiCad source:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:4687`
- Internal parser branch:
  `Expecting( "input, output, bidirectional, tri_state, passive, dot, round, diamond or rectangle" )`
- Observation:
  a hierarchical label with an invalid `shape` token fails through `kicad-cli` and surfaces only
  the generic message `Failed to load schematic`
- Severity: error
- Export behavior: fails before export

### `invalid_directive_label_shape`

- Relevant KiCad source:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:4687`
- Internal parser branch:
  `Expecting( "input, output, bidirectional, tri_state, passive, dot, round, diamond or rectangle" )`
- Observation:
  a directive label with an invalid `shape` token fails through `kicad-cli` and surfaces only the
  generic message `Failed to load schematic`
- Severity: error
- Export behavior: fails before export

### `invalid_label_at_arity`

- Relevant KiCad source:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:4648`
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.h:169`
- Internal parser branch:
  `text->SetPosition( parseXY() )`
- Observation:
  a label with a four-value `(at x y angle extra)` tuple fails through `kicad-cli` and surfaces
  only the generic message `Failed to load schematic`
- Severity: error
- Export behavior: fails before export

### `invalid_hyperlink_url`

- Relevant KiCad source:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:877`
- Observation:
  a text item with an invalid numeric `href` token fails through `kicad-cli` and surfaces only
  the generic message `Failed to load schematic`
- Severity: error
- Export behavior: fails before export

### `cannot_parse_header`

- Relevant KiCad source:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:907`
- Internal parser message:
  `Cannot parse '%s' as a header.`
- Observation:
  a schematic containing an unknown top-level object fails through `kicad-cli` and surfaces only
  the generic message `Failed to load schematic`
- Severity: error
- Export behavior: fails before export

### `bus_alias_weird_members`

- Relevant KiCad source:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:5164`
- Observation:
  a `bus_alias` whose `members` list contains a non-symbol item fails through `kicad-cli` and
  surfaces only the generic message `Failed to load schematic`
- Severity: error
- Export behavior: fails before export

### `invalid_junction_at_arity`

- Relevant KiCad source:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:4004`
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.h:169`
- Internal parser branch:
  `junction->SetPosition( parseXY() )`
- Observation:
  a junction with a three-value `(at x y extra)` tuple fails through `kicad-cli` and surfaces
  only the generic message `Failed to load schematic`
- Severity: error
- Export behavior: fails before export

### `invalid_no_connect_at_arity`

- Relevant KiCad source:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:4060`
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.h:169`
- Internal parser branch:
  `no_connect->SetPosition( parseXY() )`
- Observation:
  a no-connect marker with a three-value `(at x y extra)` tuple fails through `kicad-cli` and
  surfaces only the generic message `Failed to load schematic`
- Severity: error
- Export behavior: fails before export

### `invalid_bus_entry_at_arity`

- Relevant KiCad source:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:4098`
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.h:169`
- Internal parser branch:
  `busEntry->SetPosition( parseXY() )`
- Observation:
  a bus-entry marker with a three-value `(at x y extra)` tuple fails through `kicad-cli` and
  surfaces only the generic message `Failed to load schematic`
- Severity: error
- Export behavior: fails before export

### `invalid_bus_entry_size_arity`

- Relevant KiCad source:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:4103`
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.h:169`
- Internal parser branch:
  `busEntry->SetSize( parseXY() )`
- Observation:
  a bus-entry marker with a three-value `(size x y extra)` tuple fails through `kicad-cli` and
  surfaces only the generic message `Failed to load schematic`
- Severity: error
- Export behavior: fails before export

### `invalid_rectangle_start_arity`

- Relevant KiCad source:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:4425`
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.h:169`
- Internal parser branch:
  `rectangle->SetPosition( parseXY() )`
- Observation:
  a rectangle with a three-value `start` tuple fails through `kicad-cli` and surfaces only the
  generic message `Failed to load schematic`
- Severity: error
- Export behavior: fails before export

### `invalid_rectangle_fill_color_arity`

- Relevant KiCad source:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp`
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.h`
- Observation:
  a rectangle whose fill `color` tuple omits alpha fails through `kicad-cli` and surfaces only
  the generic message `Failed to load schematic`
- Severity: error
- Export behavior: fails before export

### `invalid_fill_color_token`

- Relevant KiCad source:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp`
- Observation:
  a fill `color` tuple containing a non-numeric token fails through `kicad-cli` and surfaces only
  the generic message `Failed to load schematic`
- Severity: error
- Export behavior: fails before export

### `invalid_stroke_width_token`

- Relevant KiCad source:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp`
- Observation:
  a stroke `width` containing a non-numeric token fails through `kicad-cli` and surfaces only the
  generic message `Failed to load schematic`
- Severity: error
- Export behavior: fails before export

### `invalid_stroke_color_arity`

- Relevant KiCad source:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp`
- Observation:
  a stroke `color` tuple that omits alpha fails through `kicad-cli` and surfaces only the generic
  message `Failed to load schematic`
- Severity: error
- Export behavior: fails before export

### `invalid_circle_center_arity`

- Relevant KiCad source:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:4366`
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.h:169`
- Internal parser branch:
  `center = parseXY()`
- Observation:
  a circle with a three-value `center` tuple fails through `kicad-cli` and surfaces only the
  generic message `Failed to load schematic`
- Severity: error
- Export behavior: fails before export

### `invalid_circle_radius_token`

- Relevant KiCad source:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:4367`
- Internal parser branch:
  `parseInternalUnits( "radius" )`
- Observation:
  a circle with a malformed non-atom `radius` token fails through `kicad-cli` and surfaces only
  the generic message `Failed to load schematic`
- Severity: error
- Export behavior: fails before export

### `invalid_arc_start_arity`

- Relevant KiCad source:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:4301`
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.h:169`
- Internal parser branch:
  `startPoint = parseXY()`
- Observation:
  an arc with a three-value `start` tuple fails through `kicad-cli` and surfaces only the generic
  message `Failed to load schematic`
- Severity: error
- Export behavior: fails before export

### `invalid_bezier_point_arity`

- Relevant KiCad source:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:4568`
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.h:169`
- Internal parser branch:
  `bezier->SetStart( parseXY() )` and sibling control-point `parseXY()` calls
- Observation:
  a Bezier curve whose first point uses a three-value `xy` tuple fails through `kicad-cli` and
  surfaces only the generic message `Failed to load schematic`
- Severity: error
- Export behavior: fails before export

### `invalid_wire_pts_arity`

- Relevant KiCad source:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:4171`
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.h:169`
- Internal parser branch:
  `polyline->AddPoint( parseXY() )`
- Observation:
  a wire whose first `xy` point uses a three-value tuple fails through `kicad-cli` and surfaces
  only the generic message `Failed to load schematic`
- Severity: error
- Export behavior: fails before export

### `invalid_polyline_pts_arity`

- Relevant KiCad source:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:4171`
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.h:169`
- Internal parser branch:
  `polyline->AddPoint( parseXY() )`
- Observation:
  a polyline whose first `xy` point uses a three-value tuple fails through `kicad-cli` and
  surfaces only the generic message `Failed to load schematic`
- Severity: error
- Export behavior: fails before export

### `invalid_bus_pts_arity`

- Relevant KiCad source:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:4171`
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.h:169`
- Internal parser branch:
  `polyline->AddPoint( parseXY() )`
- Observation:
  a bus whose first `xy` point uses a three-value tuple fails through `kicad-cli` and surfaces
  only the generic message `Failed to load schematic`
- Severity: error
- Export behavior: fails before export

### `invalid_rule_area_pt_arity`

- Relevant KiCad source:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:4171`
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.h:169`
- Internal parser branch:
  `polyline->AddPoint( parseXY() )`
- Observation:
  a rule area whose first `xy` point uses a three-value tuple fails through `kicad-cli` and
  surfaces only the generic message `Failed to load schematic`
- Severity: error
- Export behavior: fails before export

### `invalid_pin_name`

- Relevant KiCad source:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:1698`
- Observation:
  a library symbol pin with a malformed `name` token fails through `kicad-cli` and surfaces only
  the generic message `Failed to load schematic`
- Severity: error
- Export behavior: fails before export

### `invalid_pin_number`

- Relevant KiCad source:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:1738`
- Observation:
  a library symbol pin with a malformed `number` token fails through `kicad-cli` and surfaces only
  the generic message `Failed to load schematic`
- Severity: error
- Export behavior: fails before export

### `invalid_alternate_pin_name`

- Relevant KiCad source:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:1781`
- Observation:
  a library symbol pin with a malformed alternate-pin name token fails through `kicad-cli` and
  surfaces only the generic message `Failed to load schematic`
- Severity: error
- Export behavior: fails before export

### `invalid_pin_type`

- Relevant KiCad source:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:1646`
- Internal parser branch:
  `pin->SetType( parseType( token ) )`
- Observation:
  a library symbol pin with an invalid electrical type fails through `kicad-cli` and surfaces
  only the generic message `Failed to load schematic`
- Severity: error
- Export behavior: fails before export

### `valid_no_connect_pin_type`

- Relevant KiCad sources:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:1602`
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:1646`
- Internal parser branch:
  `parseType( token )` maps both `unconnected` and `no_connect` to `PT_NC`
- Observation:
  a library symbol pin using the `no_connect` electrical type exports through `kicad-cli` with
  exit 0 and no visible diagnostics
- Severity: none
- Export behavior: succeeds

### `invalid_pin_shape`

- Relevant KiCad source:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:1650`
- Internal parser branch:
  `pin->SetShape( parseShape( token ) )`
- Observation:
  a library symbol pin with an invalid graphical shape fails through `kicad-cli` and surfaces
  only the generic message `Failed to load schematic`
- Severity: error
- Export behavior: fails before export

### `invalid_alternate_pin_type`

- Relevant KiCad source:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:1788`
- Internal parser branch:
  `alt.m_Type = parseType( token )`
- Observation:
  an alternate pin entry with an invalid electrical type fails through `kicad-cli` and surfaces
  only the generic message `Failed to load schematic`
- Severity: error
- Export behavior: fails before export

### `invalid_alternate_pin_shape`

- Relevant KiCad source:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:1791`
- Internal parser branch:
  `alt.m_Shape = parseShape( token )`
- Observation:
  an alternate pin entry with an invalid graphical shape fails through `kicad-cli` and surfaces
  only the generic message `Failed to load schematic`
- Severity: error
- Export behavior: fails before export

### `invalid_table_no_cells`

- Relevant KiCad source:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:5072`
- Internal parser message:
  `Invalid table: no cells defined`
- Observation:
  a schematic table without any cell definitions fails through `kicad-cli` and surfaces only the
  generic message `Failed to load schematic`
- Severity: error
- Export behavior: fails before export

### `invalid_text_box_string`

- Relevant KiCad source:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:2034`
- Observation:
  a schematic text box with a numeric literal instead of a text string fails through `kicad-cli`
  and surfaces only the generic message `Failed to load schematic`
- Severity: error
- Export behavior: fails before export

### `valid_text_plain`

- Relevant KiCad source:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:4625`
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:4713`
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:4733`
- Internal parser path:
  plain schematic `text` accepts `at`, `effects`, and `uuid`
- Observation:
  a plain valid schematic `text` exports through `kicad-cli` with exit 0 and no visible
  diagnostics, and `ki extract` matches that accepted form
- Severity: none
- Export behavior: succeeds

### `valid_text_repeated_effects`

- Relevant KiCad source:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:4713`
- Internal parser path:
  repeated `effects` blocks on plain schematic `text` are accepted and parsed in sequence
- Observation:
  a plain schematic `text` with repeated `effects` exports through `kicad-cli` with exit 0 and no
  visible diagnostics, and `ki extract` matches that accepted form
- Severity: none
- Export behavior: succeeds

### `invalid_text_box_size_arity`

- Relevant KiCad source:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:2067`
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.h:169`
- Internal parser branch:
  `size = parseXY( true )`
- Observation:
  a text box with a three-value `(size x y extra)` tuple fails through `kicad-cli` and surfaces
  only the generic message `Failed to load schematic`
- Severity: error
- Export behavior: fails before export

### `valid_text_box_plain`

- Relevant KiCad source:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:4828`
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:4852`
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:4892`
- Internal parser path:
  plain schematic `text_box` accepts `at`, `size`, `stroke`, `fill`, `effects`, and `uuid`
- Observation:
  a plain valid schematic `text_box` exports through `kicad-cli` with exit 0 and no visible
  diagnostics, and `ki extract` matches that accepted form
- Severity: none
- Export behavior: succeeds

### `valid_text_box_margins`

- Relevant KiCad source:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:4884`
- Internal parser path:
  plain schematic `text_box` accepts explicit `margins`
- Observation:
  a plain valid schematic `text_box` with `(margins 1 2 3 4)` exports through `kicad-cli` with
  exit 0 and no visible diagnostics, and `ki extract` matches that accepted form
- Severity: none
- Export behavior: succeeds

### `valid_text_box_repeated_effects`

- Relevant KiCad source:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:4892`
- Internal parser path:
  repeated `effects` blocks on plain schematic `text_box` are accepted and parsed in sequence
- Observation:
  a plain schematic `text_box` with repeated `effects` exports through `kicad-cli` with exit 0 and
  no visible diagnostics, and `ki extract` matches that accepted form
- Severity: none
- Export behavior: succeeds

### `invalid_text_effects_font_size_arity`

- Relevant KiCad source:
  - `/Users/Daniel/Desktop/kicad/common/eda_text.cpp`
- Observation:
  a text item whose effects font `size` tuple has three numeric values fails through `kicad-cli`
  and surfaces only the generic message `Failed to load schematic`
- Severity: error
- Export behavior: fails before export

### `invalid_text_effects_font_size_token`

- Relevant KiCad source:
  - `/Users/Daniel/Desktop/kicad/common/eda_text.cpp`
- Observation:
  a text item whose effects font `size` tuple contains a non-numeric token fails through
  `kicad-cli` and surfaces only the generic message `Failed to load schematic`
- Severity: error
- Export behavior: fails before export

### `invalid_text_effects_color_arity`

- Relevant KiCad source:
  - `/Users/Daniel/Desktop/kicad/common/eda_text.cpp`
- Observation:
  a text item whose effects `color` tuple omits alpha fails through `kicad-cli` and surfaces only
  the generic message `Failed to load schematic`
- Severity: error
- Export behavior: fails before export

### `invalid_text_effects_color_token`

- Relevant KiCad source:
  - `/Users/Daniel/Desktop/kicad/common/eda_text.cpp`
- Observation:
  a text item whose effects `color` tuple contains a non-numeric token fails through `kicad-cli`
  and surfaces only the generic message `Failed to load schematic`
- Severity: error
- Export behavior: fails before export

### `invalid_effects_justify_token`

- Relevant KiCad source:
  - `/Users/Daniel/Desktop/kicad/common/eda_text.cpp`
- Observation:
  a text item whose effects `justify` list contains an unsupported token fails through
  `kicad-cli` and surfaces only the generic message `Failed to load schematic`
- Severity: error
- Export behavior: fails before export

### `invalid_effects_bold_token`

- Relevant KiCad source:
  - `/Users/Daniel/Desktop/kicad/common/eda_text.cpp`
- Observation:
  a text item whose font `bold` flag carries an invalid payload fails through `kicad-cli` and
  surfaces only the generic message `Failed to load schematic`
- Severity: error
- Export behavior: fails before export

### `invalid_effects_italic_token`

- Relevant KiCad source:
  - `/Users/Daniel/Desktop/kicad/common/eda_text.cpp`
- Observation:
  a text item whose font `italic` flag carries an invalid payload fails through `kicad-cli` and
  surfaces only the generic message `Failed to load schematic`
- Severity: error
- Export behavior: fails before export

### `invalid_text_at_arity`

- Relevant KiCad source:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:1978`
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:1979`
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.h:169`
- Internal parser branch:
  `text->SetPosition( parseXY( true ) )` followed by `parseDouble( "text angle" )`
- Observation:
  a text item with a four-value `(at x y angle extra)` tuple fails through `kicad-cli` and
  surfaces only the generic message `Failed to load schematic`
- Severity: error
- Export behavior: fails before export

### `invalid_title_block_comment_number`

- Relevant KiCad source:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:2264`
- Internal parser message:
  `Invalid title block comment number`
- Observation:
  a schematic title block with an out-of-range comment index fails through `kicad-cli` and
  surfaces only the generic message `Failed to load schematic`
- Severity: error
- Export behavior: fails before export

### `title_block_comment_float`

- Relevant KiCad source:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:2264`
- Internal parser path:
  title-block `comment` number goes through `parseInt( "comment" )`
- Observation:
  despite the apparent integer parser path, a title-block entry with `comment 1.5 "..."` exports
  through `kicad-cli sch export netlist` with exit code `0` and no visible diagnostics
- Parity implication:
  `ki extract --include-diagnostics` should remain silent and succeed for this outward case

### `title_block_comment_quoted_numeric`

- Relevant KiCad source:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:2264`
- Internal parser path:
  title-block `comment` number goes through `parseInt( "comment" )`
- Observation:
  a title-block entry with `comment "1" "..."` fails through `kicad-cli` and surfaces only the
  generic message `Failed to load schematic`
- Severity: error
- Export behavior: fails before export

### `invalid_parent_symbol_name`

- Relevant KiCad source:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:468`
- Internal parser message:
  `Invalid parent symbol name`
- Observation:
  a library symbol with a malformed `extends` name fails through `kicad-cli` and surfaces only
  the generic message `Failed to load schematic`
- Severity: error
- Export behavior: fails before export

### `invalid_symbol_unit_name`

- Relevant KiCad source:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:490`
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:504`
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:514`
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:520`
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:528`
- Internal parser messages:
  `Invalid symbol unit name`
  `Invalid symbol unit name prefix %s`
  `Invalid symbol unit name suffix %s`
  `Invalid symbol unit number %s`
  `Invalid symbol body style number %s`
- Observation:
  a library symbol child with malformed unit naming fails through `kicad-cli` and surfaces only
  the generic message `Failed to load schematic`
- Severity: error
- Export behavior: fails before export

### `invalid_symbol_unit_number`

- Relevant KiCad source:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:514`
- Internal parser message:
  `Invalid symbol unit number %s`
- Observation:
  a library symbol child with a non-numeric unit index fails through `kicad-cli` and surfaces only
  the generic message `Failed to load schematic`
- Severity: error
- Export behavior: fails before export

### `invalid_symbol_body_style_number`

- Relevant KiCad source:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:520`
- Internal parser message:
  `Invalid symbol body style number %s`
- Observation:
  a library symbol child with a non-numeric body-style suffix fails through `kicad-cli` and
  surfaces only the generic message `Failed to load schematic`
- Severity: error
- Export behavior: fails before export

### `invalid_symbol_unit_suffix`

- Relevant KiCad source:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:504`
- Internal parser message:
  `Invalid symbol unit name suffix %s`
- Observation:
  a library symbol child with too many suffix segments fails through `kicad-cli` and surfaces only
  the generic message `Failed to load schematic`
- Severity: error
- Export behavior: fails before export

### `invalid_sheet_pin_name`

- Relevant KiCad source:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:2443`
- Internal parser message:
  `Invalid sheet pin name`
- Observation:
  a schematic sheet pin without a valid name token fails through `kicad-cli` and surfaces only
  the generic message `Failed to load schematic`
- Severity: error
- Export behavior: fails before export

### `invalid_sheet_pin_type`

- Relevant KiCad source:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:2458`
- Internal parser branch:
  `Expecting( "input, output, bidirectional, tri_state, or passive" )`
- Observation:
  a schematic sheet pin with an invalid type token fails through `kicad-cli` and surfaces only
  the generic message `Failed to load schematic`
- Severity: error
- Export behavior: fails before export

### `invalid_sheet_pin_position_arity`

- Relevant KiCad source:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:2479`
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.h:169`
- Internal parser branch:
  `sheetPin->SetPosition( parseXY() )`
- Observation:
  a schematic sheet pin with a four-value `(at x y angle extra)` tuple fails through `kicad-cli`
  and surfaces only the generic message `Failed to load schematic`
- Severity: error
- Export behavior: fails before export

### `invalid_sheet_pin_orientation`

- Relevant KiCad source:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:2483`
- Internal parser branch:
  `Expecting( "0, 90, 180, or 270" )`
- Observation:
  a schematic sheet pin with a non-cardinal orientation fails through `kicad-cli` and surfaces
  only the generic message `Failed to load schematic`
- Severity: error
- Export behavior: fails before export

### `invalid_sheet_at_angle`

- Relevant KiCad source:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:3701`
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.h:169`
- Internal parser branch:
  `sheet->SetPosition( parseXY() )`
- Observation:
  a schematic sheet with a three-value `(at x y angle)` tuple fails through `kicad-cli` and
  surfaces only the generic message `Failed to load schematic`
- Severity: error
- Export behavior: fails before export

### `invalid_title_block_property_value`

- Relevant KiCad source:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:2313`
- Internal parser message:
  `Invalid property value`
- Observation:
  a title-block property with a non-string value fails through `kicad-cli` and surfaces only the
  generic message `Failed to load schematic`
- Severity: error
- Export behavior: fails before export

### `missing_reference`

- Relevant KiCad source:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr.cpp`
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp`
- Observation:
  a symbol instance missing the `Reference` property fails through `kicad-cli` and surfaces only
  the generic message `Failed to load schematic`
- Severity: error
- Export behavior: fails before export

### `missing_uuid`

- Relevant KiCad source:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp`
- Observation:
  a schematic symbol missing a `uuid` fails through `kicad-cli` and surfaces only the generic
  message `Failed to load schematic`
- Severity: error
- Export behavior: fails before export

### `annotation_errors`

- KiCad source: `/Users/Daniel/Desktop/kicad/eeschema/eeschema_jobs_handler.cpp`
- Function: `EESCHEMA_JOBS_HANDLER::JobExportNetlist`
- Trigger: `referenceList.CheckAnnotation(...) > 0`
- KiCad message:
  `Warning: schematic has annotation errors, please use the schematic editor to fix them`
- Severity: warning
- Export behavior: continues, exit code `0`

### `duplicate_sheet_names`

- KiCad source: `/Users/Daniel/Desktop/kicad/eeschema/eeschema_jobs_handler.cpp`
- Function: `EESCHEMA_JOBS_HANDLER::JobExportNetlist`
- Trigger: `erc.TestDuplicateSheetNames(false) > 0`
- KiCad message:
  `Warning: duplicate sheet names.`
- Severity: warning
- Export behavior: continues, exit code `0`

## Observed But Not Surfaced Via `kicad-cli sch export netlist`

### `recursive_sheet`

- Relevant KiCad source:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_screen.h`
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_sheet_path.cpp`
- Observation:
  a self-recursive sheet fixture exported with exit code `0` and no visible stdout/stderr diagnostics
- Parity implication:
  `ki extract --include-diagnostics` should not invent a visible diagnostic for this case unless
  KiCad CLI starts surfacing one

### `missing_library_symbol`

- Relevant KiCad source:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_screen.cpp`
- Observation:
  a schematic referencing a missing external library symbol exported with exit code `0` and no
  visible stdout/stderr diagnostics
- Parity implication:
  `ki extract --include-diagnostics` should not emit a visible diagnostic for this case in
  parity mode, even though KiCad has internal reporter strings for symbol-link failures

### `missing_child_sheet`

- Relevant KiCad source:
  - `/Users/Daniel/Desktop/kicad/eeschema/eeschema_helpers.cpp`
- Observation:
  a schematic referencing a missing child sheet exported with exit code `0` and no visible
  stdout/stderr diagnostics
- Parity implication:
  `ki extract --include-diagnostics` should remain silent for this case in parity mode

### `invalid_lib_id`

- Relevant KiCad source:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp`
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_screen.cpp`
- Observation:
  a schematic using an invalid-looking external `lib_id` exported with exit code `0` and no
  visible stdout/stderr diagnostics
- Parity implication:
  `ki extract --include-diagnostics` should remain silent for this case in parity mode

### `invalid_symbol_name`

- Relevant KiCad source:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:336`
- Internal parser message:
  `Invalid symbol name`
- Observation:
  a library symbol entry with an invalid name exported with exit code `0` and no visible
  stdout/stderr diagnostics
- Parity implication:
  `ki extract --include-diagnostics` should remain silent for this case in parity mode

### `invalid_symbol_library_id`

- Relevant KiCad source:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:3166`
- Internal parser message:
  `Invalid symbol library ID`
- Observation:
  a placed schematic symbol with an invalid `lib_id` exported with exit code `0` and no visible
  stdout/stderr diagnostics
- Parity implication:
  `ki extract --include-diagnostics` should remain silent for this case in parity mode

### `invalid_symbol_library_name`

- Relevant KiCad source:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:3125`
- Internal parser message:
  `Invalid symbol library name`
- Observation:
  a placed schematic symbol with a malformed `lib_name` token fails through `kicad-cli` and
  surfaces only the generic message `Failed to load schematic`
- Severity: error
- Export behavior: fails before export

### `invalid_symbol_library_name_numeric`

- Relevant KiCad source:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:3125`
- Internal parser message:
  `Invalid symbol library name`
- Observation:
  a placed schematic symbol with a bare numeric `lib_name` token fails through `kicad-cli` and
  surfaces only the generic message `Failed to load schematic`
- Severity: error
- Export behavior: fails before export

### `symbol_uuid_missing_value`

- Relevant KiCad source:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:3247`
- Internal parser path:
  `NeedSYMBOL()` for schematic-symbol `uuid`
- Observation:
  a schematic symbol with a missing `uuid` payload fails through `kicad-cli` and surfaces only the
  generic message `Failed to load schematic`
- Severity: error
- Export behavior: fails before export

### `symbol_uuid_numeric`

- Relevant KiCad source:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:3247`
- Internal parser path:
  `NeedSYMBOL()` for schematic-symbol `uuid`
- Observation:
  a schematic symbol with a bare numeric `uuid` payload fails through `kicad-cli` and surfaces
  only the generic message `Failed to load schematic`
- Severity: error
- Export behavior: fails before export

### `symbol_unit_missing_value`

- Relevant KiCad source:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:3198`
- Internal parser path:
  `parseInt( "symbol unit" )` for schematic-symbol `unit`
- Observation:
  a schematic symbol with a missing `unit` payload fails through `kicad-cli` and surfaces only
  the generic message `Failed to load schematic`
- Severity: error
- Export behavior: fails before export

### `symbol_unit_quoted_numeric`

- Relevant KiCad source:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:3205`
- Internal parser path:
  `parseInt( "symbol unit" )` for schematic-symbol `unit`
- Observation:
  a schematic symbol with `unit "1"` fails through `kicad-cli` and surfaces only the generic
  message `Failed to load schematic`
- Severity: error
- Export behavior: fails before export

### `symbol_unit_float`

- Relevant KiCad source:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:3205`
- Internal parser path:
  `parseInt( "symbol unit" )` for schematic-symbol `unit`
- Observation:
  despite the apparent integer parser path, a schematic symbol with `unit 1.5` exports through
  `kicad-cli sch export netlist` with exit code `0` and no visible diagnostics
- Parity implication:
  `ki extract --include-diagnostics` should remain silent and succeed for this outward case

### `symbol_body_style_missing_value`

- Relevant KiCad source:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:3204`
- Internal parser path:
  `parseInt( "symbol body style" )` for schematic-symbol `body_style`
- Observation:
  a schematic symbol with a missing `body_style` payload fails through `kicad-cli` and surfaces
  only the generic message `Failed to load schematic`
- Severity: error
- Export behavior: fails before export

### `symbol_body_style_quoted_numeric`

- Relevant KiCad source:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:3211`
- Internal parser path:
  `parseInt( "symbol body style" )` for schematic-symbol `body_style`
- Observation:
  a schematic symbol with `body_style "1"` fails through `kicad-cli` and surfaces only the
  generic message `Failed to load schematic`
- Severity: error
- Export behavior: fails before export

### `symbol_body_style_float`

- Relevant KiCad source:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:3211`
- Internal parser path:
  `parseInt( "symbol body style" )` for schematic-symbol `body_style`
- Observation:
  despite the apparent integer parser path, a schematic symbol with `body_style 1.5` exports
  through `kicad-cli sch export netlist` with exit code `0` and no visible diagnostics
- Parity implication:
  `ki extract --include-diagnostics` should remain silent and succeed for this outward case

### `invalid_symbol_at_orientation`

- Relevant KiCad source:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:3176`
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:3184`
- Internal parser path:
  `parseXY()` and symbol orientation switch for schematic-symbol `at`
- Observation:
  a schematic symbol with a malformed `at` payload fails through `kicad-cli` and surfaces only
  the generic message `Failed to load schematic`
- Severity: error
- Export behavior: fails before export

### `invalid_symbol_mirror_token`

- Relevant KiCad source:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:3191`
- Internal parser path:
  schematic-symbol `mirror` expects only `x` or `y`
- Observation:
  a schematic symbol with an invalid `mirror` token fails through `kicad-cli` and surfaces only
  the generic message `Failed to load schematic`
- Severity: error
- Export behavior: fails before export

### `symbol_pin_missing_number`

- Relevant KiCad source:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:3541`
- Internal parser path:
  schematic-symbol `pin` requires a symbol atom for the pin number via `NeedSYMBOL()`
- Observation:
  a schematic symbol with a placed `pin` missing its pin-number payload fails through
  `kicad-cli` and surfaces only the generic message `Failed to load schematic`
- Severity: error
- Export behavior: fails before export

### `symbol_pin_uuid_missing_value`

- Relevant KiCad source:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:3553`
- Internal parser path:
  nested schematic-symbol pin `uuid` requires a symbol atom via `NeedSYMBOL()`
- Observation:
  a schematic symbol with a placed `pin` whose nested `uuid` has no payload fails through
  `kicad-cli` and surfaces only the generic message `Failed to load schematic`
- Severity: error
- Export behavior: fails before export

### `symbol_pin_unknown_child`

- Relevant KiCad source:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:3568`
- Internal parser path:
  schematic-symbol `pin` accepts only nested `alternate` or `uuid` tokens
- Observation:
  a schematic symbol with an unexpected child token inside a placed `pin` fails through
  `kicad-cli` and surfaces only the generic message `Failed to load schematic`
- Severity: error
- Export behavior: fails before export

### `symbol_pin_uuid_numeric`

- Relevant KiCad source:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:3553`
- Internal parser path:
  nested schematic-symbol pin `uuid` requires a symbol atom via `NeedSYMBOL()`
- Observation:
  a schematic symbol with a placed `pin` whose nested `uuid` is numeric fails through
  `kicad-cli` and surfaces only the generic message `Failed to load schematic`
- Severity: error
- Export behavior: fails before export

### `variant_field_numeric_name`

- Relevant KiCad source:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:3450`
- Internal parser path:
  variant field `name` requires a symbol atom via `NeedSYMBOL()`
- Observation:
  a placed-symbol instance variant with a numeric `field/name` fails through `kicad-cli` and
  surfaces only the generic message `Failed to load schematic`
- Severity: error
- Export behavior: fails before export

### `variant_field_numeric_value`

- Relevant KiCad source:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:3455`
- Internal parser path:
  variant field `value` requires a symbol atom via `NeedSYMBOL()`
- Observation:
  a placed-symbol instance variant with a numeric `field/value` fails through `kicad-cli` and
  surfaces only the generic message `Failed to load schematic`
- Severity: error
- Export behavior: fails before export

### `valid_variant_field_symbols`

- Relevant KiCad source:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:3445`
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:3450`
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:3455`
- Internal parser path:
  a placed-symbol instance `variant` accepts a symbol-atom `name` plus `field/name` and
  `field/value` children parsed through `NeedSYMBOL()`
- Observation:
  a placed-symbol instance variant written as `(variant (name V1) (field (name MPN) (value X123)))`
  exports through `kicad-cli` without visible diagnostics, and `ki extract` matches that accepted form
- Severity: none
- Export behavior: exports successfully with no visible diagnostics

### `variant_field_duplicate_name`

- Relevant KiCad source:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:3449`
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:3455`
- Internal parser path:
  a placed-symbol instance `variant/field` allows repeated `name` entries and keeps parsing until
  the closing `field`
- Observation:
  a variant field written as `(field (name MPN) (name ALT) (value X123))` still exports through
  `kicad-cli` without visible diagnostics, and `ki extract` matches that accepted form
- Severity: none
- Export behavior: exports successfully with no visible diagnostics

### `variant_field_name_only`

- Relevant KiCad source:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:3449`
- Internal parser path:
  a placed-symbol instance `variant/field` accepts a `name` without a matching `value`
- Observation:
  a variant field written as `(field (name MPN))` still exports through `kicad-cli` without
  visible diagnostics, and `ki extract` matches that accepted form
- Severity: none
- Export behavior: exports successfully with no visible diagnostics

### `variant_field_value_only`

- Relevant KiCad source:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:3455`
- Internal parser path:
  a placed-symbol instance `variant/field` accepts a `value` without a matching `name`
- Observation:
  a variant field written as `(field (value X123))` still exports through `kicad-cli` without
  visible diagnostics, and `ki extract` matches that accepted form
- Severity: none
- Export behavior: exports successfully with no visible diagnostics

### `variant_field_duplicate_value`

- Relevant KiCad source:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:3455`
- Internal parser path:
  a placed-symbol instance `variant/field` allows repeated `value` entries and keeps parsing until
  the closing `field`
- Observation:
  a variant field written as `(field (name MPN) (value X123) (value ALT))` still exports through
  `kicad-cli` without visible diagnostics, and `ki extract` matches that accepted form
- Severity: none
- Export behavior: exports successfully with no visible diagnostics

### `variant_field_value_list_child`

- Relevant KiCad source:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:3455`
- Internal parser path:
  variant field `value` requires a symbol atom via `NeedSYMBOL()`
- Observation:
  a placed-symbol instance variant with a nested list in `field/value` fails through `kicad-cli`
  and surfaces only the generic message `Failed to load schematic`
- Severity: error
- Export behavior: fails before export

### `variant_field_unknown_child`

- Relevant KiCad source:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:3461`
- Internal parser path:
  a `variant/field` accepts only `name` and `value` children
- Observation:
  a placed-symbol instance variant with an unexpected child inside `field` fails through
  `kicad-cli` and surfaces only the generic message `Failed to load schematic`
- Severity: error
- Export behavior: fails before export

### `variant_unknown_child`

- Relevant KiCad source:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:3461`
- Internal parser path:
  variant accepts only `name`, boolean flags, and `field` children
- Observation:
  a placed-symbol instance variant with an unexpected child token fails through `kicad-cli` and
  surfaces only the generic message `Failed to load schematic`
- Severity: error
- Export behavior: fails before export

### `nested_instance_numeric_reference`

- Relevant KiCad source:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:3350`
- Internal parser path:
  placed-symbol nested `instances/path/reference` requires a symbol atom via `NeedSYMBOL()`
- Observation:
  a placed-symbol instance path with a numeric `reference` fails through `kicad-cli` and
  surfaces only the generic message `Failed to load schematic`
- Severity: error
- Export behavior: fails before export

### `nested_instance_unknown_child`

- Relevant KiCad source:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:3480`
- Internal parser path:
  placed-symbol nested `instances/path` accepts only `reference`, `unit`, `value`,
  `footprint`, or `variant`
- Observation:
  a placed-symbol instance path with an unexpected child token fails through `kicad-cli` and
  surfaces only the generic message `Failed to load schematic`
- Severity: error
- Export behavior: fails before export

### `nested_instance_value_missing_value`

- Relevant KiCad source:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:3360`
- Internal parser path:
  placed-symbol nested `instances/path/value` requires a symbol atom via `NeedSYMBOL()`
- Observation:
  a placed-symbol instance path with a missing `value` payload fails through `kicad-cli` and
  surfaces only the generic message `Failed to load schematic`
- Severity: error
- Export behavior: fails before export

### `nested_instance_footprint_missing_value`

- Relevant KiCad source:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:3369`
- Internal parser path:
  placed-symbol nested `instances/path/footprint` requires a symbol atom via `NeedSYMBOL()`
- Observation:
  a placed-symbol instance path with a missing `footprint` payload fails through `kicad-cli` and
  surfaces only the generic message `Failed to load schematic`
- Severity: error
- Export behavior: fails before export

### `nested_instance_unit_numeric_float`

- Relevant KiCad source:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:3355`
- Internal parser path:
  placed-symbol nested `instances/path/unit` goes through `parseInt( "symbol unit" )`
- Observation:
  despite the apparent integer parser path, a placed-symbol instance path with `unit 1.5`
  exported through `kicad-cli sch export netlist` with exit code `0` and no visible diagnostics
- Parity implication:
  `ki extract` should remain silent and succeed for this exact outward case

### `nested_instance_unit_quoted_numeric`

- Relevant KiCad source:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:3355`
- Internal parser path:
  placed-symbol nested `instances/path/unit` goes through `parseInt( "symbol unit" )`
- Observation:
  a placed-symbol instance path with `unit "1"` fails through `kicad-cli` and surfaces only the
  generic message `Failed to load schematic`
- Severity: error
- Export behavior: fails before export

### `nested_instance_value_tilde`

- Relevant KiCad source:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:3360`
- Internal parser path:
  placed-symbol nested `instances/path/value` uses `NeedSYMBOL()` and legacy `~` handling
- Observation:
  a placed-symbol instance path with `value ~` exported through `kicad-cli sch export netlist`
  with exit code `0` and no visible diagnostics
- Parity implication:
  `ki extract` should remain silent and succeed for this outward case

### `nested_instance_footprint_tilde`

- Relevant KiCad source:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:3369`
- Internal parser path:
  placed-symbol nested `instances/path/footprint` uses `NeedSYMBOL()` and legacy `~` handling
- Observation:
  a placed-symbol instance path with `footprint ~` exported through `kicad-cli sch export netlist`
  with exit code `0` and no visible diagnostics
- Parity implication:
  `ki extract` should remain silent and succeed for this outward case

### `default_instance_value_tilde`

- Relevant KiCad source:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:3277`
- Internal parser path:
  `default_instance/value` uses `NeedSYMBOL()` and legacy `~` handling
- Observation:
  a symbol `default_instance` with `value ~` exported through `kicad-cli sch export netlist`
  with exit code `0` and no visible diagnostics
- Parity implication:
  `ki extract` should remain silent and succeed for this outward case

### `default_instance_footprint_tilde`

- Relevant KiCad source:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:3286`
- Internal parser path:
  `default_instance/footprint` uses `NeedSYMBOL()` and legacy `~` handling
- Observation:
  a symbol `default_instance` with `footprint ~` exported through `kicad-cli sch export netlist`
  with exit code `0` and no visible diagnostics
- Parity implication:
  `ki extract` should remain silent and succeed for this outward case

### `default_instance_reference_tilde`

- Relevant KiCad source:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:3268`
- Internal parser path:
  `default_instance/reference` uses `NeedSYMBOL()`
- Observation:
  a symbol `default_instance` with `reference ~` exported through `kicad-cli sch export netlist`
  with exit code `0` and no visible diagnostics
- Parity implication:
  `ki extract` should remain silent and succeed for this outward case

### `default_instance_valid_only`

- Relevant KiCad source:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:3268`
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:3277`
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:3286`
- Internal parser path:
  valid placed-symbol `default_instance` data populates reference, unit, value, and footprint
  directly on the symbol even without duplicated placed-property fields
- Observation:
  a symbol that relies on valid `default_instance` fields exports through `kicad-cli sch export netlist`
  with exit code `0` and no visible diagnostics when the normal instance reference path is present
- Parity implication:
  `ki extract` must use valid `default_instance` content as a fallback source instead of failing
  load or dropping symbol metadata when placed `Value` and `Footprint` properties are absent

### `default_instance_unknown_child`

- Relevant KiCad source:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:3299`
- Internal parser path:
  `default_instance` accepts only `reference`, `unit`, `value`, or `footprint`
- Observation:
  a symbol `default_instance` with an unexpected child token fails through `kicad-cli` and
  surfaces only the generic message `Failed to load schematic`
- Severity: error
- Export behavior: fails before export

### `symbol_duplicate_uuid_child`

- Relevant KiCad source:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:3247`
- Internal parser path:
  repeated schematic-symbol `uuid` children are accepted by the outward export path
- Observation:
  a placed symbol with duplicate `uuid` children exported through `kicad-cli sch export netlist`
  with exit code `0` and no visible diagnostics
- Parity implication:
  `ki extract` should remain silent and succeed for this outward case

### `symbol_unknown_top_child`

- Relevant KiCad source:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:3578`
- Internal parser path:
  schematic symbols accept only the known top-level child tokens listed in the parser switch
- Observation:
  a placed symbol with an unexpected top-level child token fails through `kicad-cli` and
  surfaces only the generic message `Failed to load schematic`
- Severity: error
- Export behavior: fails before export

### `valid_image_fixture`

- Relevant KiCad source:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:3593`
- Internal parser path:
  top-level schematic `image` nodes are accepted and should not affect netlist export
- Observation:
  a KiCad-authored schematic with a valid embedded `image` exports through `kicad-cli sch export netlist`
  with exit code `0` and no visible diagnostics
- Parity implication:
  `ki extract` should ignore valid image nodes for extract semantics and remain silent

### `image_invalid_data_payload`

- Relevant KiCad source:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:3643`
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:3652`
- Internal parser path:
  image `data` tokens are concatenated, base64-decoded, and then passed to `ReadImageFile`
- Observation:
  malformed image payload data surfaces an extra warning line,
  `Warning: Unknown image data format.`, before the generic `Failed to load schematic`
- Severity: error
- Export behavior: fails before export

### `missing_sheet_name`

- Relevant KiCad source:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:3972`
- Internal parser message:
  `Missing sheet name property`
- Observation:
  a sheet missing its `Sheetname` property fails through `kicad-cli` and surfaces only the generic
  message `Failed to load schematic`
- Severity: error
- Export behavior: fails before export

### `missing_sheet_file`

- Relevant KiCad source:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:3978`
- Internal parser message:
  `Missing sheet file property`
- Observation:
  a sheet missing its `Sheetfile` property still exported through `kicad-cli sch export netlist`
  with exit code `0` and no visible stdout/stderr diagnostics
- Parity implication:
  `ki extract --include-diagnostics` should remain silent for this case in parity mode

### `invalid_sheet_instances_page`

- Relevant KiCad source:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:3813`
- Internal parser path:
  `NeedSYMBOL()` for `sheet_instances/path/page`
- Observation:
  a malformed `sheet_instances/page` payload fails through `kicad-cli` and surfaces only the
  generic message `Failed to load schematic`
- Severity: error
- Export behavior: fails before export

### `invalid_sheet_instances_path`

- Relevant KiCad source:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:3795`
- Internal parser path:
  `NeedSYMBOL()` for `sheet_instances/path`
- Observation:
  a malformed `sheet_instances/path` payload fails through `kicad-cli` and surfaces only the
  generic message `Failed to load schematic`
- Severity: error
- Export behavior: fails before export

### `invalid_sheet_instances_numeric_path`

- Relevant KiCad source:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:3795`
- Internal parser path:
  `NeedSYMBOL()` for `sheet_instances/path`
- Observation:
  a numeric `sheet_instances/path` token fails through `kicad-cli` and surfaces only the generic
  message `Failed to load schematic`
- Severity: error
- Export behavior: fails before export

### `invalid_sheet_instances_numeric_page`

- Relevant KiCad source:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:3813`
- Internal parser path:
  `NeedSYMBOL()` for `sheet_instances/path/page`
- Observation:
  a numeric `sheet_instances/page` token fails through `kicad-cli` and surfaces only the generic
  message `Failed to load schematic`
- Severity: error
- Export behavior: fails before export

### `invalid_symbol_instances_reference`

- Relevant KiCad source:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:2655`
- Internal parser path:
  `NeedSYMBOL()` for `symbol_instances/path/reference`
- Observation:
  a malformed symbol-instance `reference` payload fails through `kicad-cli` and surfaces only the
  generic message `Failed to load schematic`
- Severity: error
- Export behavior: fails before export

### `invalid_symbol_instances_numeric_path`

- Relevant KiCad source:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:2648`
- Internal parser path:
  `NeedSYMBOL()` for `symbol_instances/path`
- Observation:
  a numeric symbol-instance `path` token fails through `kicad-cli` and surfaces only the generic
  message `Failed to load schematic`
- Severity: error
- Export behavior: fails before export

### `invalid_symbol_instances_numeric_reference`

- Relevant KiCad source:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:2655`
- Internal parser path:
  `NeedSYMBOL()` for `symbol_instances/path/reference`
- Observation:
  a numeric symbol-instance `reference` token fails through `kicad-cli` and surfaces only the
  generic message `Failed to load schematic`
- Severity: error
- Export behavior: fails before export

### `invalid_symbol_instances_numeric_footprint`

- Relevant KiCad source:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:2677`
- Internal parser path:
  `NeedSYMBOL()` for `symbol_instances/path/footprint`
- Observation:
  a numeric symbol-instance `footprint` token fails through `kicad-cli` and surfaces only the
  generic message `Failed to load schematic`
- Severity: error
- Export behavior: fails before export

### `invalid_symbol_instances_unit`

- Relevant KiCad source:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:2661`
- Internal parser path:
  `parseInt( "symbol unit" )` for `symbol_instances/path/unit`
- Observation:
  an invalid symbol-instance `unit` payload fails through `kicad-cli` and surfaces only the
  generic message `Failed to load schematic`
- Severity: error
- Export behavior: fails before export

### `invalid_symbol_instances_numeric_value`

- Relevant KiCad source:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:2666`
- Internal parser path:
  `NeedSYMBOL()` for `symbol_instances/path/value`
- Observation:
  a numeric symbol-instance `value` token fails through `kicad-cli` and surfaces only the generic
  message `Failed to load schematic`
- Severity: error
- Export behavior: fails before export

### `symbol_instances_unit_float`

- Relevant KiCad source:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:2661`
- Internal parser path:
  top-level `symbol_instances/path/unit` goes through `parseInt( "symbol unit" )`
- Observation:
  despite the apparent integer parser path, a top-level `symbol_instances/path` entry with
  `unit 1.5` exports through `kicad-cli sch export netlist` with exit code `0` and no visible
  diagnostics
- Parity implication:
  `ki extract --include-diagnostics` should remain silent and succeed for this outward case

### `invalid_symbol_instances_value`

- Relevant KiCad source:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:2666`
- Internal parser path:
  `NeedSYMBOL()` for `symbol_instances/path/value`
- Observation:
  a malformed symbol-instance `value` payload fails through `kicad-cli` and surfaces only the
  generic message `Failed to load schematic`
- Severity: error
- Export behavior: fails before export

### `default_instance_invalid_reference`

- Relevant KiCad source:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:3266`
- Internal parser path:
  `NeedSYMBOL()` for `default_instance/reference`
- Observation:
  a malformed `default_instance/reference` payload fails through `kicad-cli` and surfaces only
  the generic message `Failed to load schematic`
- Severity: error
- Export behavior: fails before export

### `default_instance_numeric_reference`

- Relevant KiCad source:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:3266`
- Internal parser path:
  `NeedSYMBOL()` for `default_instance/reference`
- Observation:
  a numeric `default_instance/reference` token fails through `kicad-cli` and surfaces only the
  generic message `Failed to load schematic`
- Severity: error
- Export behavior: fails before export

### `default_instance_invalid_unit`

- Relevant KiCad source:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:3272`
- Internal parser path:
  `parseInt( "symbol unit" )` for `default_instance/unit`
- Observation:
  an invalid `default_instance/unit` payload fails through `kicad-cli` and surfaces only the
  generic message `Failed to load schematic`
- Severity: error
- Export behavior: fails before export

### `default_instance_unit_quoted_numeric`

- Relevant KiCad source:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:3272`
- Internal parser path:
  `default_instance/unit` goes through `parseInt( "symbol unit" )`
- Observation:
  a placed schematic symbol with `default_instance/unit "1"` fails through `kicad-cli` and
  surfaces only the generic message `Failed to load schematic`
- Severity: error
- Export behavior: fails before export

### `default_instance_invalid_value`

- Relevant KiCad source:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:3277`
- Internal parser path:
  `NeedSYMBOL()` for `default_instance/value`
- Observation:
  a malformed `default_instance/value` payload fails through `kicad-cli` and surfaces only the
  generic message `Failed to load schematic`
- Severity: error
- Export behavior: fails before export

### `default_instance_numeric_value`

- Relevant KiCad source:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:3277`
- Internal parser path:
  `NeedSYMBOL()` for `default_instance/value`
- Observation:
  a numeric `default_instance/value` token fails through `kicad-cli` and surfaces only the
  generic message `Failed to load schematic`
- Severity: error
- Export behavior: fails before export

### `default_instance_numeric_footprint`

- Relevant KiCad source:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:3288`
- Internal parser path:
  `NeedSYMBOL()` for `default_instance/footprint`
- Observation:
  a numeric `default_instance/footprint` token fails through `kicad-cli` and surfaces only the
  generic message `Failed to load schematic`
- Severity: error
- Export behavior: fails before export

### `symbol_instances_missing_project_name`

- Relevant KiCad source:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:3336`
- Internal parser path:
  `NeedSYMBOL()` for `instances/project`
- Observation:
  a missing placed-symbol project name fails through `kicad-cli` and surfaces only the generic
  message `Failed to load schematic`
- Severity: error
- Export behavior: fails before export

### `symbol_instances_numeric_project`

- Relevant KiCad source:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:3336`
- Internal parser path:
  `NeedSYMBOL()` for `instances/project`
- Observation:
  a numeric placed-symbol project name fails through `kicad-cli` and surfaces only the generic
  message `Failed to load schematic`
- Severity: error
- Export behavior: fails before export

### `symbol_instances_missing_path_value`

- Relevant KiCad source:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:3349`
- Internal parser path:
  `NeedSYMBOL()` for `instances/project/path`
- Observation:
  a missing placed-symbol instance path fails through `kicad-cli` and surfaces only the generic
  message `Failed to load schematic`
- Severity: error
- Export behavior: fails before export

### `symbol_instances_numeric_path`

- Relevant KiCad source:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:3349`
- Internal parser path:
  `NeedSYMBOL()` for `instances/project/path`
- Observation:
  a numeric placed-symbol instance path fails through `kicad-cli` and surfaces only the generic
  message `Failed to load schematic`
- Severity: error
- Export behavior: fails before export

### `invalid_page_type`

- Relevant KiCad source:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:2137`
- Internal parser message:
  `Invalid page type`
- Observation:
  a schematic with an invalid `paper` type fails through `kicad-cli` and surfaces only the
  generic message `Failed to load schematic`
- Severity: error
- Export behavior: fails before export

### `empty_sheet_pin_name`

- Relevant KiCad source:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:2451`
- Internal parser message:
  `Empty sheet pin name`
- Observation:
  a sheet pin with an empty name fails through `kicad-cli` and surfaces only the generic
  message `Failed to load schematic`
- Severity: error
- Export behavior: fails before export

### `polyline_too_few_points`

- Relevant KiCad source:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:2934`
- Internal parser message:
  `Schematic polyline has too few points`
- Observation:
  a one-point schematic polyline fails through `kicad-cli` and surfaces only the generic message
  `Failed to load schematic`
- Severity: error
- Export behavior: fails before export

### `invalid_group_library_id`

- Relevant KiCad source:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:5203`
- Internal parser message:
  `Invalid library ID`
- Observation:
  a schematic group with an invalid `lib_id` exported with exit code `0` and no visible
  stdout/stderr diagnostics
- Parity implication:
  `ki extract --include-diagnostics` should remain silent for this case in parity mode

### `invalid_group_name`

- Relevant KiCad source:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:5103`
- Observation:
  a schematic `group` with no explicit name exported with exit code `0` and no visible
  stdout/stderr diagnostics
- Parity implication:
  `ki extract --include-diagnostics` should remain silent for this case in parity mode

### `no_schematic_object_embedded_files`

- Relevant KiCad source:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:3036`
- Internal parser message:
  `No schematic object`
- Observation:
  a schematic containing a bare `embedded_files` block exported with exit code `0` and no visible
  stdout/stderr diagnostics
- Parity implication:
  `ki extract --include-diagnostics` should remain silent for this case in parity mode

### `no_schematic_object_embedded_fonts`

- Relevant KiCad source:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:3023`
- Internal parser message:
  `No schematic object`
- Observation:
  a schematic containing a top-level `embedded_fonts` block exported with exit code `0` and no
  visible stdout/stderr diagnostics
- Parity implication:
  `ki extract --include-diagnostics` should remain silent for this case in parity mode

### `invalid_font_color_token`

- Relevant KiCad source:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:832`
- Internal parser path:
  `parseInt( "red" )` inside `effects/font/color`
- Observation:
  a non-numeric red channel in `effects/font/color` fails through `kicad-cli` and surfaces only the
  generic message `Failed to load schematic`
- Severity: error
- Export behavior: fails before export

### `invalid_font_color_arity`

- Relevant KiCad source:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:832`
- Internal parser path:
  `effects/font/color` requires four numeric channels
- Observation:
  a three-value `effects/font/color` fails through `kicad-cli` and surfaces only the generic
  message `Failed to load schematic`
- Severity: error
- Export behavior: fails before export

### `invalid_font_thickness_token`

- Relevant KiCad source:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:817`
- Internal parser path:
  `parseInternalUnits( "text thickness" )`
- Observation:
  a non-numeric `effects/font/thickness` fails through `kicad-cli` and surfaces only the generic
  message `Failed to load schematic`
- Severity: error
- Export behavior: fails before export

### `invalid_font_line_spacing_token`

- Relevant KiCad source:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:841`
- Internal parser path:
  `parseDouble( "line spacing" )`
- Observation:
  a non-numeric `effects/font/line_spacing` fails through `kicad-cli` and surfaces only the
  generic message `Failed to load schematic`
- Severity: error
- Export behavior: fails before export

### `invalid_font_face_payload`

- Relevant KiCad source:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:801`
- Internal parser path:
  `NeedSYMBOL()` for `effects/font/face`
- Observation:
  a list-valued `effects/font/face` fails through `kicad-cli` and surfaces only the generic
  message `Failed to load schematic`
- Severity: error
- Export behavior: fails before export

### `invalid_hide_token`

- Relevant KiCad source:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:889`
- Internal parser path:
  `parseMaybeAbsentBool( true )` for `effects/hide`
- Observation:
  an invalid payload like `(hide maybe)` fails through `kicad-cli` and surfaces only the generic
  message `Failed to load schematic`
- Severity: error
- Export behavior: fails before export

### `bare_bold`

- Relevant KiCad source:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:821`
- Internal parser path:
  `parseMaybeAbsentBool( true )` for `effects/font/bold`
- Observation:
  bare `(bold)` is accepted by `kicad-cli`, exports successfully, and surfaces no visible
  diagnostics
- Parity implication:
  `ki extract` must not reject bare `bold`

### `bare_italic`

- Relevant KiCad source:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:826`
- Internal parser path:
  `parseMaybeAbsentBool( true )` for `effects/font/italic`
- Observation:
  bare `(italic)` is accepted by `kicad-cli`, exports successfully, and surfaces no visible
  diagnostics
- Parity implication:
  `ki extract` must not reject bare `italic`

### `bare_hide`

- Relevant KiCad source:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:889`
- Internal parser path:
  `parseMaybeAbsentBool( true )` for `effects/hide`
- Observation:
  bare `(hide)` is accepted by `kicad-cli`, exports successfully, and surfaces no visible
  diagnostics
- Parity implication:
  `ki extract` must not reject bare `hide`

### `invalid_junction_diameter_token`

- Relevant KiCad source:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:4009`
- Internal parser path:
  `parseInternalUnits( "junction diameter" )`
- Observation:
  a non-numeric `junction/diameter` fails through `kicad-cli` and surfaces only the generic
  message `Failed to load schematic`
- Severity: error
- Export behavior: fails before export

### `invalid_junction_color_token`

- Relevant KiCad source:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:4015`
- Internal parser path:
  `parseInt( "red" )` inside `junction/color`
- Observation:
  a non-numeric red channel in `junction/color` fails through `kicad-cli` and surfaces only the
  generic message `Failed to load schematic`
- Severity: error
- Export behavior: fails before export

### `invalid_junction_color_arity`

- Relevant KiCad source:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:4015`
- Internal parser path:
  `junction/color` requires four numeric channels
- Observation:
  a three-value `junction/color` fails through `kicad-cli` and surfaces only the generic
  message `Failed to load schematic`
- Severity: error
- Export behavior: fails before export

### `invalid_directive_length_token`

- Relevant KiCad source:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:4698`
- Internal parser path:
  `parseInternalUnits( "pin length" )` for directive labels
- Observation:
  a non-numeric directive-label `length` fails through `kicad-cli` and surfaces only the generic
  message `Failed to load schematic`
- Severity: error
- Export behavior: fails before export

### `invalid_text_box_margins_token`

- Relevant KiCad source:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:4878`
- Internal parser path:
  `parseMargins(...)`
- Observation:
  a non-numeric `text_box/margins` entry fails through `kicad-cli` and surfaces only the generic
  message `Failed to load schematic`
- Severity: error
- Export behavior: fails before export

### `invalid_text_box_exclude_from_sim_token`

- Relevant KiCad source:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:4828`
- Internal parser path:
  `parseBool()` for `text_box/exclude_from_sim`
- Observation:
  an invalid `text_box/exclude_from_sim` payload fails through `kicad-cli` and surfaces only the
  generic message `Failed to load schematic`
- Severity: error
- Export behavior: fails before export

### `invalid_directive_fields_autoplaced_token`

- Relevant KiCad source:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:4708`
- Internal parser path:
  `parseMaybeAbsentBool( true )` for label `fields_autoplaced`
- Observation:
  an invalid directive-label `fields_autoplaced` payload fails through `kicad-cli` and surfaces
  only the generic message `Failed to load schematic`
- Severity: error
- Export behavior: fails before export

### `invalid_directive_exclude_from_sim_token`

- Relevant KiCad source:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:4643`
- Internal parser path:
  `parseBool()` for label `exclude_from_sim`
- Observation:
  an invalid directive-label `exclude_from_sim` payload fails through `kicad-cli` and surfaces
  only the generic message `Failed to load schematic`
- Severity: error
- Export behavior: fails before export

### `label_invalid_exclude_from_sim_token`

- Relevant KiCad source:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:4643`
- Internal parser path:
  `parseBool()` for local-label `exclude_from_sim`
- Observation:
  a local `label` with malformed `exclude_from_sim` payload fails through `kicad-cli` and surfaces
  only the generic message `Failed to load schematic`
- Severity: error
- Export behavior: fails before export

### `global_label_invalid_exclude_from_sim_token`

- Relevant KiCad source:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:4643`
- Internal parser path:
  `parseBool()` for global-label `exclude_from_sim`
- Observation:
  a `global_label` with malformed `exclude_from_sim` payload fails through `kicad-cli` and
  surfaces only the generic message `Failed to load schematic`
- Severity: error
- Export behavior: fails before export

### `hier_label_invalid_exclude_from_sim_token`

- Relevant KiCad source:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:4643`
- Internal parser path:
  `parseBool()` for hierarchical-label `exclude_from_sim`
- Observation:
  a `hierarchical_label` with malformed `exclude_from_sim` payload fails through `kicad-cli` and
  surfaces only the generic message `Failed to load schematic`
- Severity: error
- Export behavior: fails before export

### `bare_directive_fields_autoplaced`

- Relevant KiCad source:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:4708`
- Internal parser path:
  `parseMaybeAbsentBool( true )` for label `fields_autoplaced`
- Observation:
  bare directive-label `fields_autoplaced` is accepted by `kicad-cli`, exports successfully, and
  surfaces no visible diagnostics
- Parity implication:
  `ki extract` must not reject bare directive-label `fields_autoplaced`

### `invalid_text_box_span_token`

- Relevant KiCad source:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:4861`
- Internal parser path:
  `parseInt( "column span" )` / `parseInt( "row span" )`
- Observation:
  a non-numeric `text_box/span` entry fails through `kicad-cli` and surfaces only the generic
  message `Failed to load schematic`
- Severity: error
- Export behavior: fails before export

### `invalid_table_column_token`

- Relevant KiCad source:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:5042`
- Internal parser path:
  `parseInt( "column width" )`
- Observation:
  a non-numeric `table/column` entry fails through `kicad-cli` and surfaces only the generic
  message `Failed to load schematic`
- Severity: error
- Export behavior: fails before export

### `invalid_table_header_token`

- Relevant KiCad source:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:5059`
- Internal parser path:
  `parseBool()` for `table/header`
- Observation:
  an invalid `table/header` payload fails through `kicad-cli` and surfaces only the generic
  message `Failed to load schematic`
- Severity: error
- Export behavior: fails before export

### `invalid_table_border_external_token`

- Relevant KiCad source:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:5008`
- Internal parser path:
  `parseBool()` for `table/border/external`
- Observation:
  an invalid `table/border/external` payload fails through `kicad-cli` and surfaces only the
  generic message `Failed to load schematic`
- Severity: error
- Export behavior: fails before export

### `invalid_table_border_header_token`

- Relevant KiCad source:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:5012`
- Internal parser path:
  `parseBool()` for `table/border/header`
- Observation:
  an invalid `table/border/header` payload fails through `kicad-cli` and surfaces only the
  generic message `Failed to load schematic`
- Severity: error
- Export behavior: fails before export

### `invalid_table_col_widths_token`

- Relevant KiCad source:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:5034`
- Internal parser path:
  `parseInt( "column width" )` entries inside `col_widths`
- Observation:
  a non-numeric `table/col_widths` entry fails through `kicad-cli` and surfaces only the generic
  message `Failed to load schematic`
- Severity: error
- Export behavior: fails before export

### `invalid_table_row_token`

- Relevant KiCad source:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:5046`
- Internal parser path:
  `parseInt( "row height" )`
- Observation:
  a non-numeric `table/row` entry fails through `kicad-cli` and surfaces only the generic
  message `Failed to load schematic`
- Severity: error
- Export behavior: fails before export

### `invalid_table_row_heights_token`

- Relevant KiCad source:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:5038`
- Internal parser path:
  `parseInt( "row height" )` entries inside `row_heights`
- Observation:
  a non-numeric `table/row_heights` entry fails through `kicad-cli` and surfaces only the generic
  message `Failed to load schematic`
- Severity: error
- Export behavior: fails before export

### `invalid_table_separators_token`

- Relevant KiCad source:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:5050`
- Internal parser path:
  `parseInt( "separator count" )`
- Observation:
  a non-numeric `table/separators` entry fails through `kicad-cli` and surfaces only the generic
  message `Failed to load schematic`
- Severity: error
- Export behavior: fails before export

### `invalid_sheet_size_token`

- Relevant KiCad source:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:3701`
- Internal parser path:
  `parseInternalUnits( "sheet width" )` / `parseInternalUnits( "sheet height" )`
- Observation:
  a non-numeric `sheet/size` entry fails through `kicad-cli` and surfaces only the generic
  message `Failed to load schematic`
- Severity: error
- Export behavior: fails before export

### `invalid_sheet_dnp_token`

- Relevant KiCad source:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:3720`
- Internal parser path:
  `parseBool()` for `sheet/dnp`
- Observation:
  an invalid `sheet/dnp` payload fails through `kicad-cli` and surfaces only the generic
  message `Failed to load schematic`
- Severity: error
- Export behavior: fails before export

### `invalid_sheet_fields_autoplaced_token`

- Relevant KiCad source:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:3724`
- Internal parser path:
  `parseMaybeAbsentBool( true )` for `sheet/fields_autoplaced`
- Observation:
  an invalid payload like `(fields_autoplaced maybe)` fails through `kicad-cli` and surfaces only
  the generic message `Failed to load schematic`
- Severity: error
- Export behavior: fails before export

### `invalid_sheet_exclude_from_sim_token`

- Relevant KiCad source:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:3709`
- Internal parser path:
  `parseBool()` for `sheet/exclude_from_sim`
- Observation:
  an invalid `sheet/exclude_from_sim` payload fails through `kicad-cli` and surfaces only the
  generic message `Failed to load schematic`
- Severity: error
- Export behavior: fails before export

### `invalid_sheet_variant_exclude_from_sim_token`

- Relevant KiCad source:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:3878`
- Internal parser path:
  `parseBool()` for sheet-instance `variant/exclude_from_sim`
- Observation:
  an invalid sheet-variant `exclude_from_sim` payload fails through `kicad-cli` and surfaces only
  the generic message `Failed to load schematic`
- Severity: error
- Export behavior: fails before export

### `bare_sheet_variant_exclude_from_sim`

- Relevant KiCad source:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:3878`
- Internal parser path:
  `parseBool()` for sheet-instance `variant/exclude_from_sim`
- Observation:
  bare sheet-variant `exclude_from_sim` also fails through `kicad-cli` and surfaces only the
  generic message `Failed to load schematic`
- Severity: error
- Export behavior: fails before export

### `valid_sheet_variant_exclude_from_sim`

- Relevant KiCad source:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:3838`
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:3878`
- Internal parser path:
  nested sheet `instances/path` accepts `page` and `variant`, and a valid
  `variant/exclude_from_sim yes` routes through `parseBool()`
- Observation:
  a sheet instance with `variant (name "V1") (exclude_from_sim yes)` exports through `kicad-cli`
  with exit code `0` and no visible diagnostics
- Parity implication:
  `ki extract` must accept the sheet-instance `page or variant` grammar instead of applying the
  symbol-instance child set there

### `valid_sheet_variant_field_symbols`

- Relevant KiCad source:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:3838`
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:3916`
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:3921`
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:3926`
- Internal parser path:
  nested sheet `instances/path` accepts `page` and `variant`, and a sheet-instance
  `variant/field/name|value` uses `NeedSYMBOL()`
- Observation:
  a sheet instance variant written as `(variant (name V1) (field (name MPN) (value X123)))`
  exports through `kicad-cli` with exit code `0` and no visible diagnostics, and `ki extract`
  matches that accepted form
- Severity: none
- Export behavior: exports successfully with no visible diagnostics

### `sheet_variant_field_value_list_child`

- Relevant KiCad source:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:3926`
- Internal parser path:
  sheet-instance variant field `value` requires a symbol atom via `NeedSYMBOL()`
- Observation:
  a sheet instance variant with a nested list in `field/value` fails through `kicad-cli` and
  surfaces only the generic message `Failed to load schematic`
- Severity: error
- Export behavior: fails before export

### `sheet_variant_numeric_name`

- Relevant KiCad source:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:3916`
- Internal parser path:
  sheet-instance `variant/name` requires a non-numeric atom via `NeedSYMBOL()`
- Observation:
  a sheet instance variant with numeric `name` fails through `kicad-cli` and surfaces only the
  generic message `Failed to load schematic`
- Severity: error
- Export behavior: fails before export

### `sheet_variant_field_numeric_name`

- Relevant KiCad source:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:3921`
- Internal parser path:
  sheet-instance variant field `name` requires a non-numeric atom via `NeedSYMBOL()`
- Observation:
  a sheet instance variant with numeric `field/name` fails through `kicad-cli` and surfaces only
  the generic message `Failed to load schematic`
- Severity: error
- Export behavior: fails before export

### `sheet_variant_field_numeric_value`

- Relevant KiCad source:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:3926`
- Internal parser path:
  sheet-instance variant field `value` requires a non-numeric atom via `NeedSYMBOL()`
- Observation:
  a sheet instance variant with numeric `field/value` fails through `kicad-cli` and surfaces only
  the generic message `Failed to load schematic`
- Severity: error
- Export behavior: fails before export

### `sheet_variant_field_name_list_child`

- Relevant KiCad source:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:3932`
- Internal parser path:
  a sheet-instance `variant/field` rejects a nested list in place of the field name atom
- Observation:
  `(field (name (foo)) (value X123))` inside a sheet variant fails through `kicad-cli` and
  surfaces only the generic message `Failed to load schematic`
- Severity: error
- Export behavior: fails before export

### `sheet_variant_field_duplicate_name`

- Relevant KiCad source:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:3921`
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:3926`
- Internal parser path:
  a sheet-instance `variant/field` allows repeated `name` entries and keeps parsing until the
  closing `field`
- Observation:
  a sheet variant field written as `(field (name MPN) (name ALT) (value X123))` still exports
  through `kicad-cli` without visible diagnostics, and `ki extract` matches that accepted form
- Severity: none
- Export behavior: exports successfully with no visible diagnostics

### `sheet_variant_field_value_only`

- Relevant KiCad source:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:3926`
- Internal parser path:
  a sheet-instance `variant/field` accepts a `value` without a matching `name`
- Observation:
  a sheet variant field written as `(field (value X123))` still exports through `kicad-cli`
  without visible diagnostics, and `ki extract` matches that accepted form
- Severity: none
- Export behavior: exports successfully with no visible diagnostics

### `sheet_variant_field_duplicate_value`

- Relevant KiCad source:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:3926`
- Internal parser path:
  a sheet-instance `variant/field` allows repeated `value` entries and keeps parsing until the
  closing `field`
- Observation:
  a sheet variant field written as `(field (name MPN) (value X123) (value ALT))` still exports
  through `kicad-cli` without visible diagnostics, and `ki extract` matches that accepted form
- Severity: none
- Export behavior: exports successfully with no visible diagnostics

### `sheet_variant_unknown_child`

- Relevant KiCad source:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:3932`
- Internal parser path:
  a sheet-instance `variant` accepts only `name`, boolean flags, and `field` children
- Observation:
  a sheet instance variant with an unexpected child fails through `kicad-cli` and surfaces only
  the generic message `Failed to load schematic`
- Severity: error
- Export behavior: fails before export

### `sheet_variant_extra_bare_atom`

- Relevant KiCad source:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:3885`
- Internal parser path:
  a sheet-instance `variant` requires list children and rejects a trailing bare atom via
  `Expecting( T_LEFT )`
- Observation:
  `(variant (name V1) foo)` inside a sheet instance fails through `kicad-cli` and surfaces only
  the generic message `Failed to load schematic`
- Severity: error
- Export behavior: fails before export

### `invalid_sheet_variant_in_pos_files_token`

- Relevant KiCad source:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:3909`
- Internal parser path:
  `parseBool()` for sheet-instance `variant/in_pos_files`
- Observation:
  an invalid sheet-variant `in_pos_files` payload fails through `kicad-cli` and surfaces only the
  generic message `Failed to load schematic`
- Severity: error
- Export behavior: fails before export

### `bare_sheet_variant_in_pos_files`

- Relevant KiCad source:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:3909`
- Internal parser path:
  `parseBool()` for sheet-instance `variant/in_pos_files`
- Observation:
  bare sheet-variant `in_pos_files` also fails through `kicad-cli` and surfaces only the generic
  message `Failed to load schematic`
- Severity: error
- Export behavior: fails before export

### `valid_sheet_variant_in_pos_files`

- Relevant KiCad source:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:3838`
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:3909`
- Internal parser path:
  nested sheet `instances/path` accepts `page` and `variant`, and a valid
  `variant/in_pos_files yes` routes through `parseBool()`
- Observation:
  a sheet instance with `variant (name V1) (in_pos_files yes)` exports through `kicad-cli`
  with exit code `0` and no visible diagnostics
- Severity: none
- Export behavior: exports successfully with no visible diagnostics

### `invalid_sheet_variant_dnp_token`

- Relevant KiCad source:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:3890`
- Internal parser path:
  `parseBool()` for sheet-instance `variant/dnp`
- Observation:
  an invalid sheet-variant `dnp` payload fails through `kicad-cli` and surfaces only the generic
  message `Failed to load schematic`
- Severity: error
- Export behavior: fails before export

### `bare_sheet_variant_dnp`

- Relevant KiCad source:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:3890`
- Internal parser path:
  `parseBool()` for sheet-instance `variant/dnp`
- Observation:
  bare sheet-variant `dnp` also fails through `kicad-cli` and surfaces only the generic message
  `Failed to load schematic`
- Severity: error
- Export behavior: fails before export

### `valid_sheet_variant_dnp`

- Relevant KiCad source:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:3838`
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:3890`
- Internal parser path:
  nested sheet `instances/path` accepts `page` and `variant`, and a valid `variant/dnp yes`
  routes through `parseBool()`
- Observation:
  a sheet instance with `variant (name V1) (dnp yes)` exports through `kicad-cli` with exit code
  `0` and no visible diagnostics
- Severity: none
- Export behavior: exports successfully with no visible diagnostics

### `invalid_sheet_variant_in_bom_token`

- Relevant KiCad source:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:3894`
- Internal parser path:
  `parseBool()` for sheet-instance `variant/in_bom`
- Observation:
  an invalid sheet-variant `in_bom` payload fails through `kicad-cli` and surfaces only the
  generic message `Failed to load schematic`
- Severity: error
- Export behavior: fails before export

### `bare_sheet_variant_in_bom`

- Relevant KiCad source:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:3894`
- Internal parser path:
  `parseBool()` for sheet-instance `variant/in_bom`
- Observation:
  bare sheet-variant `in_bom` also fails through `kicad-cli` and surfaces only the generic
  message `Failed to load schematic`
- Severity: error
- Export behavior: fails before export

### `valid_sheet_variant_in_bom`

- Relevant KiCad source:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:3838`
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:3894`
- Internal parser path:
  nested sheet `instances/path` accepts `page` and `variant`, and a valid `variant/in_bom yes`
  routes through `parseBool()`
- Observation:
  a sheet instance with `variant (name V1) (in_bom yes)` exports through `kicad-cli` with exit code
  `0` and no visible diagnostics
- Severity: none
- Export behavior: exports successfully with no visible diagnostics

### `invalid_sheet_variant_on_board_token`

- Relevant KiCad source:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:3902`
- Internal parser path:
  `parseBool()` for sheet-instance `variant/on_board`
- Observation:
  an invalid sheet-variant `on_board` payload fails through `kicad-cli` and surfaces only the
  generic message `Failed to load schematic`
- Severity: error
- Export behavior: fails before export

### `bare_sheet_variant_on_board`

- Relevant KiCad source:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:3902`
- Internal parser path:
  `parseBool()` for sheet-instance `variant/on_board`
- Observation:
  bare sheet-variant `on_board` also fails through `kicad-cli` and surfaces only the generic
  message `Failed to load schematic`
- Severity: error
- Export behavior: fails before export

### `valid_sheet_variant_on_board`

- Relevant KiCad source:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:3838`
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:3902`
- Internal parser path:
  nested sheet `instances/path` accepts `page` and `variant`, and a valid `variant/on_board yes`
  routes through `parseBool()`
- Observation:
  a sheet instance with `variant (name V1) (on_board yes)` exports through `kicad-cli` with exit
  code `0` and no visible diagnostics
- Severity: none
- Export behavior: exports successfully with no visible diagnostics

### `invalid_sheet_in_bom_token`

- Relevant KiCad source:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:3713`
- Internal parser path:
  `parseBool()` for `sheet/in_bom`
- Observation:
  an invalid `sheet/in_bom` payload fails through `kicad-cli` and surfaces only the generic
  message `Failed to load schematic`
- Severity: error
- Export behavior: fails before export

### `invalid_sheet_on_board_token`

- Relevant KiCad source:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:3717`
- Internal parser path:
  `parseBool()` for `sheet/on_board`
- Observation:
  an invalid `sheet/on_board` payload fails through `kicad-cli` and surfaces only the generic
  message `Failed to load schematic`
- Severity: error
- Export behavior: fails before export

### `bare_sheet_fields_autoplaced`

- Relevant KiCad source:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:3724`
- Internal parser path:
  `parseMaybeAbsentBool( true )` for `sheet/fields_autoplaced`
- Observation:
  bare `(fields_autoplaced)` on a sheet is accepted by `kicad-cli`, exports successfully, and
  surfaces no visible diagnostics
- Parity implication:
  `ki extract` must not reject bare sheet `fields_autoplaced`

### `invalid_symbol_dnp_token`

- Relevant KiCad source:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:3237`
- Internal parser path:
  `parseBool()` for schematic-symbol `dnp`
- Observation:
  an invalid symbol `dnp` payload fails through `kicad-cli` and surfaces only the generic
  message `Failed to load schematic`
- Severity: error
- Export behavior: fails before export

### `invalid_symbol_exclude_from_sim_token`

- Relevant KiCad source:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:3216`
- Internal parser path:
  `parseBool()` for schematic-symbol `exclude_from_sim`
- Observation:
  an invalid symbol `exclude_from_sim` payload fails through `kicad-cli` and surfaces only the
  generic message `Failed to load schematic`
- Severity: error
- Export behavior: fails before export

### `invalid_symbol_in_bom_token`

- Relevant KiCad source:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:3229`
- Internal parser path:
  `parseBool()` for schematic-symbol `in_bom`
- Observation:
  an invalid symbol `in_bom` payload fails through `kicad-cli` and surfaces only the generic
  message `Failed to load schematic`
- Severity: error
- Export behavior: fails before export

### `invalid_symbol_on_board_token`

- Relevant KiCad source:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:3233`
- Internal parser path:
  `parseBool()` for schematic-symbol `on_board`
- Observation:
  an invalid symbol `on_board` payload fails through `kicad-cli` and surfaces only the generic
  message `Failed to load schematic`
- Severity: error
- Export behavior: fails before export

### `invalid_symbol_in_pos_files_token`

- Relevant KiCad source:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:3225`
- Internal parser path:
  `parseBool()` for schematic-symbol `in_pos_files`
- Observation:
  an invalid symbol `in_pos_files` payload fails through `kicad-cli` and surfaces only the
  generic message `Failed to load schematic`
- Severity: error
- Export behavior: fails before export

### `invalid_symbol_fields_autoplaced_token`

- Relevant KiCad source:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:3241`
- Internal parser path:
  `parseMaybeAbsentBool( true )` for schematic-symbol `fields_autoplaced`
- Observation:
  an invalid payload like `(fields_autoplaced maybe)` fails through `kicad-cli` and surfaces only
  the generic message `Failed to load schematic`
- Severity: error
- Export behavior: fails before export

### `invalid_rule_area_exclude_from_sim_token`

- Relevant KiCad source:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:4505`
- Internal parser path:
  `parseBool()` for `rule_area/exclude_from_sim`
- Observation:
  an invalid `rule_area/exclude_from_sim` payload fails through `kicad-cli` and surfaces only the
  generic message `Failed to load schematic`
- Severity: error
- Export behavior: fails before export

### `valid_rule_area_exclude_from_sim`

- Relevant KiCad source:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:4505`
- Internal parser path:
  `parseBool()` for `rule_area/exclude_from_sim`
- Observation:
  a valid `rule_area/exclude_from_sim yes` payload exports through `kicad-cli` with exit 0 and no
  visible diagnostics
- Severity: none
- Export behavior: succeeds

### `bare_rule_area_exclude_from_sim`

- Relevant KiCad source:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:4505`
- Internal parser path:
  `parseBool()` for `rule_area/exclude_from_sim`
- Observation:
  a bare `rule_area/exclude_from_sim` without a boolean value fails through `kicad-cli` and
  surfaces only the generic message `Failed to load schematic`
- Severity: error
- Export behavior: fails before export

### `invalid_rule_area_dnp_token`

- Relevant KiCad source:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:4520`
- Internal parser path:
  `parseBool()` for `rule_area/dnp`
- Observation:
  an invalid `rule_area/dnp` payload fails through `kicad-cli` and surfaces only the generic
  message `Failed to load schematic`
- Severity: error
- Export behavior: fails before export

### `valid_rule_area_dnp`

- Relevant KiCad source:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:4520`
- Internal parser path:
  `parseBool()` for `rule_area/dnp`
- Observation:
  a valid `rule_area/dnp yes` payload exports through `kicad-cli` with exit 0 and no visible
  diagnostics
- Severity: none
- Export behavior: succeeds

### `bare_rule_area_dnp`

- Relevant KiCad source:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:4520`
- Internal parser path:
  `parseBool()` for `rule_area/dnp`
- Observation:
  a bare `rule_area/dnp` without a boolean value fails through `kicad-cli` and surfaces only the
  generic message `Failed to load schematic`
- Severity: error
- Export behavior: fails before export

### `valid_rule_area_plain`

- Relevant KiCad source:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:4478`
- Internal parser path:
  `parseSchRuleArea()` requires a child `polyline`
- Observation:
  a plain valid `rule_area` with only a `polyline` exports through `kicad-cli` with exit 0 and no
  visible diagnostics
- Severity: none
- Export behavior: succeeds

### `invalid_rule_area_in_bom_token`

- Relevant KiCad source:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:4510`
- Internal parser path:
  `parseBool()` for `rule_area/in_bom`
- Observation:
  an invalid `rule_area/in_bom` payload fails through `kicad-cli` and surfaces only the generic
  message `Failed to load schematic`
- Severity: error
- Export behavior: fails before export

### `bare_rule_area_in_bom`

- Relevant KiCad source:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:4510`
- Internal parser path:
  `parseBool()` for `rule_area/in_bom`
- Observation:
  a bare `rule_area/in_bom` without a boolean value fails through `kicad-cli` and surfaces only the
  generic message `Failed to load schematic`
- Severity: error
- Export behavior: fails before export

### `valid_rule_area_in_bom`

- Relevant KiCad source:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:4510`
- Internal parser path:
  `parseBool()` for `rule_area/in_bom`
- Observation:
  a valid `rule_area/in_bom` payload exports through `kicad-cli` with exit 0 and no visible
  diagnostics
- Severity: none
- Export behavior: succeeds

### `invalid_rule_area_on_board_token`

- Relevant KiCad source:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:4515`
- Internal parser path:
  `parseBool()` for `rule_area/on_board`
- Observation:
  an invalid `rule_area/on_board` payload fails through `kicad-cli` and surfaces only the generic
  message `Failed to load schematic`
- Severity: error
- Export behavior: fails before export

### `bare_rule_area_on_board`

- Relevant KiCad source:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:4515`
- Internal parser path:
  `parseBool()` for `rule_area/on_board`
- Observation:
  a bare `rule_area/on_board` without a boolean value fails through `kicad-cli` and surfaces only
  the generic message `Failed to load schematic`
- Severity: error
- Export behavior: fails before export

### `valid_rule_area_on_board`

- Relevant KiCad source:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:4515`
- Internal parser path:
  `parseBool()` for `rule_area/on_board`
- Observation:
  a valid `rule_area/on_board` payload exports through `kicad-cli` with exit 0 and no visible
  diagnostics
- Severity: none
- Export behavior: succeeds

### `invalid_lib_symbol_exclude_from_sim_token`

- Relevant KiCad source:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:405`
- Internal parser path:
  `parseBool()` for library-symbol `exclude_from_sim`
- Observation:
  an invalid library-symbol `exclude_from_sim` payload fails through `kicad-cli` and surfaces only
  the generic message `Failed to load schematic`
- Severity: error
- Export behavior: fails before export

### `invalid_lib_symbol_fonts_embedded_token`

- Relevant KiCad source:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:601`
- Internal parser path:
  `parseBool()` for library-symbol `embedded_fonts`
- Observation:
  an invalid library-symbol `embedded_fonts` payload fails through `kicad-cli` and surfaces only
  the generic message `Failed to load schematic`
- Severity: error
- Export behavior: fails before export

### `invalid_lib_symbol_jumpers_token`

- Relevant KiCad source:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:425`
- Internal parser path:
  `parseBool()` for `duplicate_pin_numbers_are_jumpers`
- Observation:
  an invalid library-symbol `duplicate_pin_numbers_are_jumpers` payload fails through `kicad-cli`
  and surfaces only the generic message `Failed to load schematic`
- Severity: error
- Export behavior: fails before export

### `bare_symbol_fields_autoplaced`

- Relevant KiCad source:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:3241`
- Internal parser path:
  `parseMaybeAbsentBool( true )` for schematic-symbol `fields_autoplaced`
- Observation:
  bare `(fields_autoplaced)` on a placed symbol is accepted by `kicad-cli`, exports successfully,
  and surfaces no visible diagnostics
- Parity implication:
  `ki extract` must not reject bare symbol `fields_autoplaced`

### `invalid_pin_names_offset_token`

- Relevant KiCad source:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:982`
- Internal parser path:
  `parseInternalUnits( "pin name offset" )`
- Observation:
  a non-numeric `pin_names/offset` fails through `kicad-cli` and surfaces only the generic
  message `Failed to load schematic`
- Severity: error
- Export behavior: fails before export

### `invalid_pin_names_hide_token`

- Relevant KiCad source:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:986`
- Internal parser path:
  `parseBool()` for `pin_names/hide`
- Observation:
  an invalid `pin_names/hide` payload fails through `kicad-cli` and surfaces only the generic
  message `Failed to load schematic`
- Severity: error
- Export behavior: fails before export

### `invalid_pin_numbers_hide_token`

- Relevant KiCad source:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:1034`
- Internal parser path:
  `parseBool()` for `pin_numbers/hide`
- Observation:
  an invalid `pin_numbers/hide` payload fails through `kicad-cli` and surfaces only the generic
  message `Failed to load schematic`
- Severity: error
- Export behavior: fails before export

### `invalid_pin_hide_token`

- Relevant KiCad source:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:1689`
- Internal parser path:
  `parseBool()` for library-symbol pin `hide`
- Observation:
  an invalid pin `hide` payload fails through `kicad-cli` and surfaces only the generic message
  `Failed to load schematic`
- Severity: error
- Export behavior: fails before export

### `bare_pin_names_hide`

- Relevant KiCad source:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:971`
- Internal parser path:
  pre-20241004 bare `hide` form inside `pin_names`
- Observation:
  bare `pin_names hide` is accepted by `kicad-cli`, exports successfully, and surfaces no visible
  diagnostics
- Parity implication:
  `ki extract` must not reject bare `pin_names hide`

### `bare_pin_numbers_hide`

- Relevant KiCad source:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:1018`
- Internal parser path:
  pre-20241004 bare `hide` form inside `pin_numbers`
- Observation:
  bare `pin_numbers hide` is accepted by `kicad-cli`, exports successfully, and surfaces no
  visible diagnostics
- Parity implication:
  `ki extract` must not reject bare `pin_numbers hide`

### `bare_pin_hide`

- Relevant KiCad source:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:1689`
- Internal parser path:
  `parseBool()` for library-symbol pin `hide`
- Observation:
  bare pin `hide` fails through `kicad-cli` and surfaces only the generic message
  `Failed to load schematic`
- Severity: error
- Export behavior: fails before export

### `invalid_show_name_token`

- Relevant KiCad source:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:1143`
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:2411`
- Internal parser path:
  `parseMaybeAbsentBool( true )` for property `show_name`
- Observation:
  an invalid property `show_name` payload fails through `kicad-cli` and surfaces only the generic
  message `Failed to load schematic`
- Severity: error
- Export behavior: fails before export

### `bare_show_name`

- Relevant KiCad source:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:1143`
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:2411`
- Internal parser path:
  bare property `show_name` routes through `parseMaybeAbsentBool( true )`
- Observation:
  bare property `show_name` is accepted through `kicad-cli` with no visible diagnostics
- Severity: none
- Export behavior: exports successfully

### `invalid_text_exclude_from_sim_token`

- Relevant KiCad source:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:4643`
- Internal parser path:
  `parseBool()` for plain text `exclude_from_sim`
- Observation:
  an invalid plain-text `exclude_from_sim` payload fails through `kicad-cli` and surfaces only
  the generic message `Failed to load schematic`
- Severity: error
- Export behavior: fails before export

### `bare_text_exclude_from_sim`

- Relevant KiCad source:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:4643`
- Internal parser path:
  `parseBool()` for plain text `exclude_from_sim`
- Observation:
  bare plain-text `exclude_from_sim` also fails through `kicad-cli` and surfaces only the generic
  message `Failed to load schematic`
- Severity: error
- Export behavior: fails before export

### `label_bare_exclude_from_sim`

- Relevant KiCad source:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:4643`
- Internal parser path:
  bare `exclude_from_sim` on a local `label` still routes through `parseBool()`
- Observation:
  bare local-label `exclude_from_sim` fails through `kicad-cli` and surfaces only the generic
  message `Failed to load schematic`
- Severity: error
- Export behavior: fails before export

### `global_label_bare_exclude_from_sim`

- Relevant KiCad source:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:4643`
- Internal parser path:
  bare `exclude_from_sim` on `global_label` still routes through `parseBool()`
- Observation:
  bare `global_label` `exclude_from_sim` fails through `kicad-cli` and surfaces only the generic
  message `Failed to load schematic`
- Severity: error
- Export behavior: fails before export

### `hier_label_bare_exclude_from_sim`

- Relevant KiCad source:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:4643`
- Internal parser path:
  bare `exclude_from_sim` on `hierarchical_label` still routes through `parseBool()`
- Observation:
  bare `hierarchical_label` `exclude_from_sim` fails through `kicad-cli` and surfaces only the
  generic message `Failed to load schematic`
- Severity: error
- Export behavior: fails before export

### `directive_label_bare_exclude_from_sim`

- Relevant KiCad source:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:4643`
- Internal parser path:
  bare `exclude_from_sim` on `directive_label` still routes through `parseBool()`
- Observation:
  bare `directive_label` `exclude_from_sim` fails through `kicad-cli` and surfaces only the
  generic message `Failed to load schematic`
- Severity: error
- Export behavior: fails before export

### `invalid_text_fields_autoplaced_token`

- Relevant KiCad source:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:4708`
- Internal parser path:
  `parseMaybeAbsentBool( true )` for plain text `fields_autoplaced`
- Observation:
  an invalid plain-text `fields_autoplaced` payload fails through `kicad-cli` and surfaces only
  the generic message `Failed to load schematic`
- Severity: error
- Export behavior: fails before export

### `bare_text_fields_autoplaced`

- Relevant KiCad source:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:4708`
- Internal parser path:
  bare plain-text `fields_autoplaced` routes through `parseMaybeAbsentBool( true )`
- Observation:
  bare plain-text `fields_autoplaced` is accepted through `kicad-cli` with no visible diagnostics
- Severity: none
- Export behavior: exports successfully

### `label_invalid_fields_autoplaced_token`

- Relevant KiCad source:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:4708`
- Internal parser path:
  `parseMaybeAbsentBool( true )` for local-label `fields_autoplaced`
- Observation:
  a local `label` with malformed `fields_autoplaced` payload fails through `kicad-cli` and
  surfaces only the generic message `Failed to load schematic`
- Severity: error
- Export behavior: fails before export

### `bare_label_fields_autoplaced`

- Relevant KiCad source:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:4708`
- Internal parser path:
  bare local-label `fields_autoplaced` routes through `parseMaybeAbsentBool( true )`
- Observation:
  bare local-label `fields_autoplaced` is accepted through `kicad-cli` with no visible diagnostics
- Severity: none
- Export behavior: exports successfully

### `global_label_invalid_fields_autoplaced_token`

- Relevant KiCad source:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:4708`
- Internal parser path:
  `parseMaybeAbsentBool( true )` for global-label `fields_autoplaced`
- Observation:
  a `global_label` with malformed `fields_autoplaced` payload fails through `kicad-cli` and
  surfaces only the generic message `Failed to load schematic`
- Severity: error
- Export behavior: fails before export

### `bare_global_label_fields_autoplaced`

- Relevant KiCad source:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:4708`
- Internal parser path:
  bare global-label `fields_autoplaced` routes through `parseMaybeAbsentBool( true )`
- Observation:
  bare global-label `fields_autoplaced` is accepted through `kicad-cli` with no visible diagnostics
- Severity: none
- Export behavior: exports successfully

### `hier_label_invalid_fields_autoplaced_token`

- Relevant KiCad source:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:4708`
- Internal parser path:
  `parseMaybeAbsentBool( true )` for hierarchical-label `fields_autoplaced`
- Observation:
  a `hierarchical_label` with malformed `fields_autoplaced` payload fails through `kicad-cli` and
  surfaces only the generic message `Failed to load schematic`
- Severity: error
- Export behavior: fails before export

### `bare_hier_label_fields_autoplaced`

- Relevant KiCad source:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:4708`
- Internal parser path:
  bare hierarchical-label `fields_autoplaced` routes through `parseMaybeAbsentBool( true )`
- Observation:
  bare hierarchical-label `fields_autoplaced` is accepted through `kicad-cli` with no visible
  diagnostics
- Severity: none
- Export behavior: exports successfully

### `invalid_do_not_autoplace_token`

- Relevant KiCad source:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:1150`
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:2418`
- Internal parser path:
  `parseMaybeAbsentBool( true )` for property `do_not_autoplace`
- Observation:
  an invalid property `do_not_autoplace` payload fails through `kicad-cli` and surfaces only the
  generic message `Failed to load schematic`
- Severity: error
- Export behavior: fails before export

### `bare_do_not_autoplace`

- Relevant KiCad source:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:1150`
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:2418`
- Internal parser path:
  bare property `do_not_autoplace` routes through `parseMaybeAbsentBool( true )`
- Observation:
  bare property `do_not_autoplace` is accepted through `kicad-cli` with no visible diagnostics
- Severity: none
- Export behavior: exports successfully

### `invalid_top_embedded_fonts_token`

- Relevant KiCad source:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:3026`
- Internal parser path:
  `parseBool()` for top-level `embedded_fonts`
- Observation:
  an invalid top-level `embedded_fonts` payload fails through `kicad-cli` and surfaces only the
  generic message `Failed to load schematic`
- Severity: error
- Export behavior: fails before export

### `bare_top_embedded_fonts`

- Relevant KiCad source:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:3026`
- Internal parser path:
  `parseBool()` for top-level `embedded_fonts`
- Observation:
  bare top-level `embedded_fonts` also fails through `kicad-cli` and surfaces only the generic
  message `Failed to load schematic`
- Severity: error
- Export behavior: fails before export

### `invalid_variant_dnp_token`

- Relevant KiCad source:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:3402`
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:3873`
- Internal parser path:
  `parseBool()` for `variant/dnp` inside symbol or sheet instances
- Observation:
  an invalid variant `dnp` payload fails through `kicad-cli` and surfaces only the generic
  message `Failed to load schematic`
- Severity: error
- Export behavior: fails before export

### `invalid_variant_in_bom_token`

- Relevant KiCad source:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:3412`
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:3883`
- Internal parser path:
  `parseBool()` for `variant/in_bom` inside symbol or sheet instances
- Observation:
  an invalid variant `in_bom` payload fails through `kicad-cli` and surfaces only the generic
  message `Failed to load schematic`
- Severity: error
- Export behavior: fails before export

### `invalid_variant_in_pos_files_token`

- Relevant KiCad source:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:3441`
- Internal parser path:
  `parseBool()` for schematic-symbol `variant/in_pos_files`
- Observation:
  an invalid symbol-variant `in_pos_files` payload fails through `kicad-cli` and surfaces only the
  generic message `Failed to load schematic`
- Severity: error
- Export behavior: fails before export

### `bare_variant_in_pos_files`

- Relevant KiCad source:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:3441`
- Internal parser path:
  `parseBool()` for schematic-symbol `variant/in_pos_files`
- Observation:
  bare symbol-variant `in_pos_files` also fails through `kicad-cli` and surfaces only the generic
  message `Failed to load schematic`
- Severity: error
- Export behavior: fails before export

### `valid_variant_in_pos_files`

- Relevant KiCad source:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:3441`
- Internal parser path:
  a valid schematic-symbol `variant/in_pos_files yes` routes through `parseBool()`
- Observation:
  a placed symbol with `variant (name V1) (in_pos_files yes)` exports through `kicad-cli`
  with exit code `0` and no visible diagnostics
- Severity: none
- Export behavior: exports successfully with no visible diagnostics

### `invalid_variant_on_board_token`

- Relevant KiCad source:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:3439`
- Internal parser path:
  `parseBool()` for schematic-symbol `variant/on_board`
- Observation:
  an invalid symbol-variant `on_board` payload fails through `kicad-cli` and surfaces only the
  generic message `Failed to load schematic`
- Severity: error
- Export behavior: fails before export

### `bare_variant_on_board`

- Relevant KiCad source:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:3439`
- Internal parser path:
  `parseBool()` for schematic-symbol `variant/on_board`
- Observation:
  bare symbol-variant `on_board` also fails through `kicad-cli` and surfaces only the generic
- message `Failed to load schematic`
- Severity: error
- Export behavior: fails before export

### `valid_variant_on_board`

- Relevant KiCad source:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:3439`
- Internal parser path:
  a valid schematic-symbol `variant/on_board yes` routes through `parseBool()`
- Observation:
  a placed symbol with `variant (name V1) (on_board yes)` exports through `kicad-cli`
  with exit code `0` and no visible diagnostics
- Severity: none
- Export behavior: exports successfully with no visible diagnostics

### `invalid_variant_name`

- Relevant KiCad source:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:3397`
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:3868`
- Internal parser path:
  `NeedSYMBOL()` for `variant/name`
- Observation:
  a malformed variant `name` payload fails through `kicad-cli` and surfaces only the generic
  message `Failed to load schematic`
- Severity: error
- Export behavior: fails before export

### `invalid_variant_field_name`

- Relevant KiCad source:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:3444`
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:3915`
- Internal parser path:
  `NeedSYMBOL()` for `variant/field/name`
- Observation:
  a malformed variant field name fails through `kicad-cli` and surfaces only the generic message
  `Failed to load schematic`
- Severity: error
- Export behavior: fails before export

## Internal Error Catalog

The items below are useful for parity research even when they are not directly surfaced by
`kicad-cli sch export netlist`. They come from the local KiCad source and are grouped by the
layer where they originate.

### Parser / Loader Messages That Commonly Collapse To `Failed to load schematic`

- `Invalid symbol name`
- `Invalid library identifier`
- `Invalid symbol library name`
- `Invalid symbol library ID`
- `Invalid property name`
- `Empty property name`
- `Invalid property value`
- `Invalid text string`
- `Invalid hyperlink url '%s'`
- `Invalid pin name`
- `Invalid pin number`
- `Invalid alternate pin name`
- pin `parseType(...)` failures
- pin `parseShape(...)` failures
- `Invalid text string`
- `Invalid page type`
- `Invalid title block comment number`
- `Invalid sheet pin name`
- `Empty sheet pin name`
- `Schematic polyline has too few points`
- `No schematic object`
- `Invalid table: no cells defined`
- `Failed to read image data.`

### Parser / Loader Messages Observed But Not Surfaced By `kicad-cli sch export netlist`

- `Missing sheet name property`
- `Missing sheet file property`
- `Schematic polyline has too few points`
- `Invalid symbol name`
- `Invalid symbol library ID`
- `Invalid library ID`
- `No schematic object`

### Symbol Linker Reporter Messages In KiCad Internals

These come from `SCH_SCREEN::UpdateSymbolLinks` in
`/Users/Daniel/Desktop/kicad/eeschema/sch_screen.cpp`. The current export CLI probes did not
surface them, but they are relevant if parity work later expands beyond the current CLI output
surface.

- `Schematic symbol reference '%s' library identifier is not valid. Unable to link library symbol.`
- `Symbol library '%s' not found and no fallback cache library available.  Unable to link library symbol.`
- `I/O error %s resolving library symbol %s`
- `Falling back to cache to set symbol '%s:%s' link '%s'.`
- `No library symbol found for schematic symbol '%s %s'.`

## Formal Source Audit

This section records a direct source walk of the KiCad surfaces that define the current
`kicad-cli sch export netlist --format kicadxml` oracle used by `ki extract` parity.

Audited source entrypoints:

- CLI wrapper:
  - `/Users/Daniel/Desktop/kicad/kicad/cli/command_sch_export_netlist.cpp`
- Job/export path:
  - `/Users/Daniel/Desktop/kicad/eeschema/eeschema_jobs_handler.cpp`
  - `/Users/Daniel/Desktop/kicad/eeschema/eeschema.cpp`
- Reference / duplicate-sheet pre-export checks:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_reference_list.cpp`
  - `/Users/Daniel/Desktop/kicad/eeschema/erc/erc.cpp`
- KiCad s-expression parser / loader:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp`

Audit result:

- Top-level CLI behavior is accounted for.
  - `missing_input_file` covers the explicit CLI missing-file branch.
  - invalid `--format` is intentionally out of scope because `ki extract` does not expose the
    KiCad `--format` switch; parity uses only `kicadxml` as the oracle.
- Job-handler level behavior is accounted for.
  - `failed_to_load_schematic` covers the generic load failure surfaced by KiCad.
  - `annotation_errors` covers the `SCH_REFERENCE_LIST::CheckAnnotation(...)` warning path.
  - `duplicate_sheet_names` covers the pre-export duplicate-sheet warning path.
- Parser branch families discovered in the source walk are now explicitly accounted for.
  - `invalid_lib_symbol_power_scope_token` isolates `Expecting( "global or local" )`
  - `invalid_jumper_pin_group_member` isolates `Expecting( "list of pin names" )`
  - `invalid_lib_symbols_child` isolates `Expecting( "symbol, generator, or generator_version" )`
  - `invalid_pin_names_child` isolates `Expecting( "offset or hide" )`
  - `bus_alias_numeric_member` isolates `Expecting( "quoted string" )`
  - `bus_alias_missing_members` isolates `Expecting( "members" )`
  - `invalid_group_header_token` isolates `Expecting( "group name or locked" )`
  - `invalid_group_child_token` isolates `Expecting( "uuid, lib_id, members" )`
- Existing malformed-input fixtures continue to cover the broader parser surface.
  - Invalid property/name/value payloads
  - Invalid text/effects payloads
  - Invalid geometry tuple arities
  - Invalid symbol / sheet / instance payloads
  - Missing required fields like `uuid`, `reference`, `sheetfile`, `sheetname`
  - Various “bare” boolean/list forms that KiCad accepts or silently ignores

Formal conclusion:

- `tests/extract_parity` is now source-exhaustive for the current
  `kicad-cli sch export netlist --format kicadxml` oracle surface.
- Every distinct relevant branch found in the audited KiCad CLI/job/parser path is now either:
  - represented by a dedicated parity fixture,
  - represented by an equivalent existing parity fixture,
  - or explicitly marked out of scope because `ki extract` does not expose that KiCad CLI surface.

## Mutation Discovery

To keep searching beyond direct source-branch inventory, the repo now also includes a differential
mutation harness:

- Test: `/Users/Daniel/Desktop/modular/tools/ki/tests/cli_extract_mutation.rs`
- Method:
  - start from a known-good extract parity seed schematic
  - apply one deterministic mutation at a time
  - run both `kicad-cli sch export netlist --format kicadxml` and `ki extract`
  - fail on any exit-code or message mismatch
- Current mutation families:
  - optional enum token corruption
  - optional child kind corruption
  - quoted-vs-unquoted token class changes
  - missing structural keywords
  - header token kind corruption
  - legacy bare-form acceptance
  - payload decoding failures
  - sentinel token handling like `~`

This harness is intended to catch cases that a direct source walk can still miss, especially
token-boundary and interaction bugs.

## Additional Numeric Token-Class Cases

### `symbol_at_quoted_angle`

- Relevant KiCad sources:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:3176`
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:3184`
- Internal parser path:
  placed-symbol `at` uses `parseXY()` and a symbol-orientation switch
- Observation:
  a placed schematic symbol with `(at x y "0")` fails through `kicad-cli` and surfaces only the
  generic message `Failed to load schematic`
- Severity: error
- Export behavior: fails before export

### `pin_length_quoted_numeric`

- Relevant KiCad source:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:1684`
- Internal parser path:
  library-symbol `pin/length` goes through `parseInternalUnits( "pin length" )`
- Observation:
  a library symbol pin with `(length "1.27")` fails through `kicad-cli` and surfaces only the
  generic message `Failed to load schematic`
- Severity: error
- Export behavior: fails before export

### `pin_names_offset_quoted_numeric`

- Relevant KiCad source:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:982`
- Internal parser path:
  `pin_names/offset` goes through `parseInternalUnits( "pin name offset" )`
- Observation:
  a library symbol `pin_names` block with `(offset "0")` fails through `kicad-cli` and surfaces
  only the generic message `Failed to load schematic`
- Severity: error
- Export behavior: fails before export

### `stroke_width_quoted_numeric`

- Relevant KiCad source:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:696`
- Internal parser path:
  `parseStroke(...)` for stroke `width`
- Observation:
  a stroke with `(width "0.254")` fails through `kicad-cli` and surfaces only the generic
  message `Failed to load schematic`
- Severity: error
- Export behavior: fails before export

### `text_effects_font_size_quoted_numeric`

- Relevant KiCad source:
  - `/Users/Daniel/Desktop/kicad/common/eda_text.cpp`
- Internal parser path:
  text effects font `size` parsing requires numeric tokens for both dimensions
- Observation:
  a text item whose effects font `size` tuple contains a quoted numeric token fails through
  `kicad-cli` and surfaces only the generic message `Failed to load schematic`
- Severity: error
- Export behavior: fails before export

### `junction_diameter_quoted_numeric`

- Relevant KiCad source:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:4009`
- Internal parser path:
  `parseInternalUnits( "junction diameter" )`
- Observation:
  a junction with `(diameter "1.27")` fails through `kicad-cli` and surfaces only the generic
  message `Failed to load schematic`
- Severity: error
- Export behavior: fails before export

### `duplicate_effects_color`

- Relevant KiCad source:
  - `/Users/Daniel/Desktop/kicad/common/eda_text.cpp`
- Internal parser path:
  top-level `effects` accepts a single `color` child for text items
- Observation:
  a text item with duplicate top-level `effects/color` children fails through `kicad-cli` and
  surfaces only the generic message `Failed to load schematic`
- Severity: error
- Export behavior: fails before export

### `root_uuid_list`

- Relevant KiCad source:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:942`
- Internal parser path:
  top-level schematic `uuid` expects a single atom payload
- Observation:
  a root schematic `uuid` encoded as a nested list fails through `kicad-cli` and surfaces only
  the generic message `Failed to load schematic`
- Severity: error
- Export behavior: fails before export

### `root_uuid_numeric`

- Relevant KiCad source:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:942`
- Internal parser path:
  top-level schematic `uuid` expects a non-numeric atom payload
- Observation:
  a root schematic `uuid` encoded as a bare numeric token fails through `kicad-cli` and
  surfaces only the generic message `Failed to load schematic`
- Severity: error
- Export behavior: fails before export

### `junction_uuid_list`

- Relevant KiCad sources:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:4029`
- Internal parser path:
  schematic junction UUID parsing requires a symbol token
- Observation:
  a junction `uuid` given as a list fails through `kicad-cli` and surfaces only the generic
  message `Failed to load schematic`
- Severity: error
- Export behavior: fails before export

### `junction_uuid_numeric`

- Relevant KiCad sources:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:4029`
- Internal parser path:
  schematic junction UUID parsing requires a symbol token
- Observation:
  a junction `uuid` given as a numeric token fails through `kicad-cli` and surfaces only the
  generic message `Failed to load schematic`
- Severity: error
- Export behavior: fails before export

### `no_connect_uuid_list`

- Relevant KiCad sources:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:4066`
- Internal parser path:
  no-connect UUID parsing requires a symbol token
- Observation:
  a `no_connect` `uuid` given as a list fails through `kicad-cli` and surfaces only the generic
  message `Failed to load schematic`
- Severity: error
- Export behavior: fails before export

### `no_connect_uuid_numeric`

- Relevant KiCad sources:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:4066`
- Internal parser path:
  no-connect UUID parsing requires a symbol token
- Observation:
  a `no_connect` `uuid` given as a numeric token fails through `kicad-cli` and surfaces only the
  generic message `Failed to load schematic`
- Severity: error
- Export behavior: fails before export

### `bus_entry_uuid_list`

- Relevant KiCad sources:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:4120`
- Internal parser path:
  bus-entry UUID parsing requires a symbol token
- Observation:
  a `bus_entry` `uuid` given as a list fails through `kicad-cli` and surfaces only the generic
  message `Failed to load schematic`
- Severity: error
- Export behavior: fails before export

### `bus_entry_uuid_numeric`

- Relevant KiCad sources:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:4120`
- Internal parser path:
  bus-entry UUID parsing requires a symbol token
- Observation:
  a `bus_entry` `uuid` given as a numeric token fails through `kicad-cli` and surfaces only the
  generic message `Failed to load schematic`
- Severity: error
- Export behavior: fails before export

### `wire_uuid_list`

- Relevant KiCad sources:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:4265`
- Internal parser path:
  wire UUID parsing requires a symbol token
- Observation:
  a `wire` `uuid` given as a list fails through `kicad-cli` and surfaces only the generic
  message `Failed to load schematic`
- Severity: error
- Export behavior: fails before export

### `wire_uuid_numeric`

- Relevant KiCad sources:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:4265`
- Internal parser path:
  wire UUID parsing requires a symbol token
- Observation:
  a `wire` `uuid` given as a numeric token fails through `kicad-cli` and surfaces only the
  generic message `Failed to load schematic`
- Severity: error
- Export behavior: fails before export

### `polyline_uuid_list`

- Relevant KiCad sources:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:4196`
- Internal parser path:
  polyline UUID parsing requires a symbol token
- Observation:
  a `polyline` `uuid` given as a list fails through `kicad-cli` and surfaces only the generic
  message `Failed to load schematic`
- Severity: error
- Export behavior: fails before export

### `polyline_uuid_numeric`

- Relevant KiCad sources:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:4196`
- Internal parser path:
  polyline UUID parsing requires a symbol token
- Observation:
  a `polyline` `uuid` given as a numeric token fails through `kicad-cli` and surfaces only the
  generic message `Failed to load schematic`
- Severity: error
- Export behavior: fails before export

### `bus_uuid_list`

- Relevant KiCad sources:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:4265`
- Internal parser path:
  bus UUID parsing requires a symbol token
- Observation:
  a `bus` `uuid` given as a list fails through `kicad-cli` and surfaces only the generic message
  `Failed to load schematic`
- Severity: error
- Export behavior: fails before export

### `bus_uuid_numeric`

- Relevant KiCad sources:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:4265`
- Internal parser path:
  bus UUID parsing requires a symbol token
- Observation:
  a `bus` `uuid` given as a numeric token fails through `kicad-cli` and surfaces only the generic
  message `Failed to load schematic`
- Severity: error
- Export behavior: fails before export

### `arc_uuid_list`

- Relevant KiCad sources:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:4329`
- Internal parser path:
  arc UUID parsing requires a symbol token
- Observation:
  an `arc` `uuid` given as a list fails through `kicad-cli` and surfaces only the generic message
  `Failed to load schematic`
- Severity: error
- Export behavior: fails before export

### `arc_uuid_numeric`

- Relevant KiCad sources:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:4329`
- Internal parser path:
  arc UUID parsing requires a symbol token
- Observation:
  an `arc` `uuid` given as a numeric token fails through `kicad-cli` and surfaces only the
  generic message `Failed to load schematic`
- Severity: error
- Export behavior: fails before export

### `circle_uuid_list`

- Relevant KiCad sources:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:4389`
- Internal parser path:
  circle UUID parsing requires a symbol token
- Observation:
  a `circle` `uuid` given as a list fails through `kicad-cli` and surfaces only the generic
  message `Failed to load schematic`
- Severity: error
- Export behavior: fails before export

### `circle_uuid_numeric`

- Relevant KiCad sources:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:4389`
- Internal parser path:
  circle UUID parsing requires a symbol token
- Observation:
  a `circle` `uuid` given as a numeric token fails through `kicad-cli` and surfaces only the
  generic message `Failed to load schematic`
- Severity: error
- Export behavior: fails before export

### `rectangle_uuid_list`

- Relevant KiCad sources:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:4453`
- Internal parser path:
  rectangle UUID parsing requires a symbol token
- Observation:
  a `rectangle` `uuid` given as a list fails through `kicad-cli` and surfaces only the generic
  message `Failed to load schematic`
- Severity: error
- Export behavior: fails before export

### `rectangle_uuid_numeric`

- Relevant KiCad sources:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:4453`
- Internal parser path:
  rectangle UUID parsing requires a symbol token
- Observation:
  a `rectangle` `uuid` given as a numeric token fails through `kicad-cli` and surfaces only the
  generic message `Failed to load schematic`
- Severity: error
- Export behavior: fails before export

### `bezier_uuid_list`

- Relevant KiCad sources:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:4594`
- Internal parser path:
  bezier UUID parsing requires a symbol token
- Observation:
  a `bezier` `uuid` given as a list fails through `kicad-cli` and surfaces only the generic
  message `Failed to load schematic`
- Severity: error
- Export behavior: fails before export

### `bezier_uuid_numeric`

- Relevant KiCad sources:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:4594`
- Internal parser path:
  bezier UUID parsing requires a symbol token
- Observation:
  a `bezier` `uuid` given as a numeric token fails through `kicad-cli` and surfaces only the
  generic message `Failed to load schematic`
- Severity: error
- Export behavior: fails before export

### `text_uuid_list`

- Relevant KiCad sources:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:4735`
- Internal parser path:
  schematic text UUID parsing requires a symbol token
- Observation:
  a `text` `uuid` given as a list fails through `kicad-cli` and surfaces only the generic
  message `Failed to load schematic`
- Severity: error
- Export behavior: fails before export

### `text_uuid_numeric`

- Relevant KiCad sources:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:4735`
- Internal parser path:
  schematic text UUID parsing requires a symbol token
- Observation:
  a `text` `uuid` given as a numeric token fails through `kicad-cli` and surfaces only the
  generic message `Failed to load schematic`
- Severity: error
- Export behavior: fails before export

### `duplicate_table_header`

- Relevant KiCad source:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:4999`
- Internal parser path:
  table header visibility is a single boolean child
- Observation:
  a table with duplicate `headers` children fails through `kicad-cli` and surfaces only the
  generic message `Failed to load schematic`
- Severity: error
- Export behavior: fails before export

### `duplicate_table_stroke`

- Relevant KiCad source:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:5014`
- Internal parser path:
  table border stroke is a single nested `stroke` block
- Observation:
  a table with duplicate top-level `stroke` children fails through `kicad-cli` and surfaces only
  the generic message `Failed to load schematic`
- Severity: error
- Export behavior: fails before export

### `empty_pts_then_valid`

- Relevant KiCad source:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:4171`
- Internal parser path:
  every `pts` block for wire-like items must contain valid `xy` children
- Observation:
  a wire containing an empty `pts` block followed by a valid `pts` block still fails through
  `kicad-cli` and surfaces only the generic message `Failed to load schematic`
- Severity: error
- Export behavior: fails before export

### `valid_pts_then_empty`

- Relevant KiCad source:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:4171`
- Internal parser path:
  every `pts` block for wire-like items must contain valid `xy` children
- Observation:
  a wire containing a valid `pts` block followed by an empty `pts` block still fails through
  `kicad-cli` and surfaces only the generic message `Failed to load schematic`
- Severity: error
- Export behavior: fails before export

### `duplicate_nested_variant`

- Relevant KiCad source:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:3461`
- Internal parser path:
  placed-symbol nested `instances/path` accepts repeated `variant` children
- Observation:
  a placed-symbol instance path with two valid `variant` children exports through `kicad-cli sch export netlist`
  with exit code `0` and no visible diagnostics
- Parity implication:
  `ki extract --include-diagnostics` should remain silent and succeed for this outward case

### `global_label_with_directive_length`

- Relevant KiCad source:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:4702`
- Internal parser path:
  directive-label-only `length` child is not valid on `global_label`
- Observation:
  a `global_label` carrying `length` fails through `kicad-cli` and surfaces only the generic
  message `Failed to load schematic`
- Severity: error
- Export behavior: fails before export

### `hier_label_with_directive_length`

- Relevant KiCad source:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:4702`
- Internal parser path:
  directive-label-only `length` child is not valid on `hierarchical_label`
- Observation:
  a `hierarchical_label` carrying `length` fails through `kicad-cli` and surfaces only the generic
  message `Failed to load schematic`
- Severity: error
- Export behavior: fails before export

### `text_with_label_shape`

- Relevant KiCad sources:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:1962`
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:4687`
- Internal parser path:
  label-only `shape` child is not valid on plain `text`
- Observation:
  a `text` item carrying `shape input` fails through `kicad-cli` and surfaces only the generic
  message `Failed to load schematic`
- Severity: error
- Export behavior: fails before export

### `duplicate_text_href`

- Relevant KiCad source:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:877`
- Internal parser path:
  plain `text` accepts at most one `href` child
- Observation:
  a `text` item with duplicate `href` children fails through `kicad-cli` and surfaces only the
  generic message `Failed to load schematic`
- Severity: error
- Export behavior: fails before export

### `label_with_uuid_and_href`

- Relevant KiCad sources:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:1962`
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:877`
- Internal parser path:
  plain `label` does not accept the combined `uuid`/`href` child shape used in richer objects
- Observation:
  a `label` carrying both `uuid` and `href` fails through `kicad-cli` and surfaces only the
  generic message `Failed to load schematic`
- Severity: error
- Export behavior: fails before export

### `text_box_with_shape`

- Relevant KiCad sources:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:4687`
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:4903`
- Internal parser path:
  label-style `shape` is not valid on `text_box`
- Observation:
  a `text_box` carrying `shape input` fails through `kicad-cli` and surfaces only the generic
  message `Failed to load schematic`
- Severity: error
- Export behavior: fails before export

### `text_box_with_href`

- Relevant KiCad sources:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:768`
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:4903`
- Internal parser path:
  `href` is only accepted inside `effects`-parsed text objects; a direct `href` child is not valid
  on `text_box`
- Observation:
  a `text_box` carrying a direct `href` child fails through `kicad-cli` and surfaces only the
  generic message `Failed to load schematic`
- Severity: error
- Export behavior: fails before export

### `image_uuid_numeric`

- Relevant KiCad source:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:3624`
- Internal parser path:
  image `uuid` uses `NeedSYMBOL` plus `parseKIID`, so bare numeric tokens are rejected
- Observation:
  an `image` with a numeric `uuid` fails through `kicad-cli` and surfaces only the generic
  message `Failed to load schematic`
- Severity: error
- Export behavior: fails before export

### `image_uuid_list`

- Relevant KiCad source:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:3626`
- Internal parser path:
  image `uuid` uses `NeedSYMBOL` plus `parseKIID`, so list payloads are rejected
- Observation:
  an `image` with a list `uuid` fails through `kicad-cli` and surfaces only the generic message
  `Failed to load schematic`
- Severity: error
- Export behavior: fails before export

### `symbol_uuid_list`

- Relevant KiCad source:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:3248`
- Internal parser path:
  placed symbol `uuid` uses `NeedSYMBOL` plus `parseKIID`, so list payloads are rejected
- Observation:
  a placed `symbol` with a list `uuid` fails through `kicad-cli` and surfaces only the generic
  message `Failed to load schematic`
- Severity: error
- Export behavior: fails before export

### `image_with_href`

- Relevant KiCad source:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:3658`
- Internal parser path:
  image objects only accept `at`, `scale`, `uuid`, and `data` children
- Observation:
  an `image` carrying direct `href` fails through `kicad-cli` and surfaces only the generic
  message `Failed to load schematic`
- Severity: error
- Export behavior: fails before export

### `symbol_property_with_href`

- Relevant KiCad source:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:2425`
- Internal parser path:
  symbol property fields only accept `at`, visibility, autoplace flags, and `effects`
- Observation:
  a placed symbol property carrying direct `href` fails through `kicad-cli` and surfaces only the
  generic message `Failed to load schematic`
- Severity: error
- Export behavior: fails before export

### `sheet_property_with_href`

- Relevant KiCad source:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:2425`
- Internal parser path:
  sheet property fields only accept `at`, visibility, autoplace flags, and `effects`
- Observation:
  a sheet property carrying direct `href` fails through `kicad-cli` and surfaces only the generic
  message `Failed to load schematic`
- Severity: error
- Export behavior: fails before export

### `lib_symbol_property_with_href`

- Relevant KiCad source:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:2425`
- Internal parser path:
  library symbol property fields only accept `at`, visibility, autoplace flags, and `effects`
- Observation:
  a library symbol property carrying direct `href` fails through `kicad-cli` and surfaces only the
  generic message `Failed to load schematic`
- Severity: error
- Export behavior: fails before export

### `lib_pin_name_with_href`

- Relevant KiCad source:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:1727`
- Internal parser path:
  library pin names only accept an optional trailing `effects` block
- Observation:
  a library pin name carrying direct `href` fails through `kicad-cli` and surfaces only the
  generic message `Failed to load schematic`
- Severity: error
- Export behavior: fails before export

### `lib_pin_number_with_href`

- Relevant KiCad source:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:1767`
- Internal parser path:
  library pin numbers only accept an optional trailing `effects` block
- Observation:
  a library pin number carrying direct `href` fails through `kicad-cli` and surfaces only the
  generic message `Failed to load schematic`
- Severity: error
- Export behavior: fails before export

### `text_with_href`

- Relevant KiCad source:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:4762`
- Internal parser path:
  plain schematic text only accepts `at`, `uuid`, and `effects`
- Observation:
  a `text` item carrying direct `href` fails through `kicad-cli` and surfaces only the generic
  message `Failed to load schematic`
- Severity: error
- Export behavior: fails before export

### `label_with_href`

- Relevant KiCad source:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:4762`
- Internal parser path:
  local labels do not accept direct `href` children
- Observation:
  a `label` carrying direct `href` fails through `kicad-cli` and surfaces only the generic
  message `Failed to load schematic`
- Severity: error
- Export behavior: fails before export

### `global_label_with_href`

- Relevant KiCad source:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:4762`
- Internal parser path:
  global labels do not accept direct `href` children
- Observation:
  a `global_label` carrying direct `href` fails through `kicad-cli` and surfaces only the generic
  message `Failed to load schematic`
- Severity: error
- Export behavior: fails before export

### `hier_label_with_href`

- Relevant KiCad source:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:4762`
- Internal parser path:
  hierarchical labels do not accept direct `href` children
- Observation:
  a `hierarchical_label` carrying direct `href` fails through `kicad-cli` and surfaces only the
  generic message `Failed to load schematic`
- Severity: error
- Export behavior: fails before export

### `directive_label_with_href`

- Relevant KiCad source:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:4762`
- Internal parser path:
  directive labels do not accept direct `href` children
- Observation:
  a `directive_label` carrying direct `href` fails through `kicad-cli` and surfaces only the
  generic message `Failed to load schematic`
- Severity: error
- Export behavior: fails before export

### `text_box_effects_href`

- Relevant KiCad sources:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:768`
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:4894`
- Internal parser path:
  `text_box` delegates its `effects` block to `parseEDA_TEXT`, but the schematic textbox path does
  not accept hyperlink-bearing text the way plain `text` does
- Observation:
  a `text_box` whose `effects` block contains `href` fails through `kicad-cli` and surfaces only
  the generic message `Failed to load schematic`
- Severity: error
- Export behavior: fails before export

### `text_box_effects_duplicate_href`

- Relevant KiCad sources:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:768`
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:4894`
- Internal parser path:
  schematic `text_box` `effects` parsing also rejects duplicate `href` entries
- Observation:
  a schematic `text_box` whose `effects` block contains duplicate `href` entries fails through
  `kicad-cli` and surfaces only the generic message `Failed to load schematic`
- Severity: error
- Export behavior: fails before export

### `text_box_uuid_numeric`

- Relevant KiCad source:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:4898`
- Internal parser path:
  `text_box` `uuid` uses `NeedSYMBOL` plus `parseKIID`, so bare numeric tokens are rejected
- Observation:
  a `text_box` with a numeric `uuid` fails through `kicad-cli` and surfaces only the generic
  message `Failed to load schematic`
- Severity: error
- Export behavior: fails before export

### `text_box_uuid_list`

- Relevant KiCad source:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:4898`
- Internal parser path:
  `text_box` `uuid` requires an atom; list-shaped payloads fail before export
- Observation:
  a `text_box` with a list-shaped `uuid` fails through `kicad-cli` and surfaces only the generic
  message `Failed to load schematic`
- Severity: error
- Export behavior: fails before export

### `table_cell_with_href`

- Relevant KiCad source:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:4903`
- Internal parser path:
  `table_cell` shares the textbox parser shape and does not accept a direct `href` child
- Observation:
  a `table_cell` carrying a direct `href` child fails through `kicad-cli` and surfaces only the
  generic message `Failed to load schematic`
- Severity: error
- Export behavior: fails before export

### `table_cell_effects_href`

- Relevant KiCad sources:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:768`
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:4894`
- Internal parser path:
  `table_cell` `effects` parsing rejects hyperlink-bearing text in the same way as schematic
  textboxes
- Observation:
  a `table_cell` whose `effects` block contains `href` fails through `kicad-cli` and surfaces
  only the generic message `Failed to load schematic`
- Severity: error
- Export behavior: fails before export

### `table_cell_effects_duplicate_href`

- Relevant KiCad sources:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:768`
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:4894`
- Internal parser path:
  `table_cell` `effects` parsing also rejects duplicate `href` entries
- Observation:
  a `table_cell` whose `effects` block contains duplicate `href` entries fails through
  `kicad-cli` and surfaces only the generic message `Failed to load schematic`
- Severity: error
- Export behavior: fails before export

### `table_cell_with_shape`

- Relevant KiCad source:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:4906`
- Internal parser path:
  `table_cell` does not accept label-style `shape` children
- Observation:
  a `table_cell` carrying `shape input` fails through `kicad-cli` and surfaces only the generic
  message `Failed to load schematic`
- Severity: error
- Export behavior: fails before export

### `table_cell_duplicate_effects`

- Relevant KiCad source:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:4895`
- Internal parser path:
  `table_cell` accepts at most one `effects` block
- Observation:
  a `table_cell` with duplicate `effects` children fails through `kicad-cli` and surfaces only
  the generic message `Failed to load schematic`
- Severity: error
- Export behavior: fails before export

### `table_cell_duplicate_uuid`

- Relevant KiCad source:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:4898`
- Internal parser path:
  `table_cell` accepts at most one `uuid` child
- Observation:
  a `table_cell` with duplicate `uuid` children fails through `kicad-cli` and surfaces only the
  generic message `Failed to load schematic`
- Severity: error
- Export behavior: fails before export

### `table_duplicate_cells_block`

- Relevant KiCad source:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:4995`
- Internal parser path:
  table parsing expects a single `cells` block
- Observation:
  a `table` with duplicate `cells` children fails through `kicad-cli` and surfaces only the
  generic message `Failed to load schematic`
- Severity: error
- Export behavior: fails before export

### `table_duplicate_rows`

- Relevant KiCad source:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:5066`
- Internal parser path:
  table parsing does not accept repeated `row` children
- Observation:
  a `table` with duplicate `row` children fails through `kicad-cli` and surfaces only the generic
  message `Failed to load schematic`
- Severity: error
- Export behavior: fails before export

### `table_duplicate_columns`

- Relevant KiCad source:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:5066`
- Internal parser path:
  table parsing does not accept repeated `column` children
- Observation:
  a `table` with duplicate `column` children fails through `kicad-cli` and surfaces only the
  generic message `Failed to load schematic`
- Severity: error
- Export behavior: fails before export

### `table_duplicate_row_heights`

- Relevant KiCad source:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:4967`
- Internal parser path:
  table parsing accepts at most one `row_heights` block
- Observation:
  a `table` with duplicate `row_heights` children fails through `kicad-cli` and surfaces only the
  generic message `Failed to load schematic`
- Severity: error
- Export behavior: fails before export

### `table_duplicate_col_widths`

- Relevant KiCad source:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:4957`
- Internal parser path:
  table parsing accepts at most one `col_widths` block
- Observation:
  a `table` with duplicate `col_widths` children fails through `kicad-cli` and surfaces only the
  generic message `Failed to load schematic`
- Severity: error
- Export behavior: fails before export

### `table_duplicate_border`

- Relevant KiCad source:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:4993`
- Internal parser path:
  table parsing accepts at most one `border` block
- Observation:
  a `table` with duplicate `border` children fails through `kicad-cli` and surfaces only the
  generic message `Failed to load schematic`
- Severity: error
- Export behavior: fails before export

### `table_duplicate_separators`

- Relevant KiCad source:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:5026`
- Internal parser path:
  table parsing accepts at most one `separators` block
- Observation:
  a `table` with duplicate `separators` children fails through `kicad-cli` and surfaces only the
  generic message `Failed to load schematic`
- Severity: error
- Export behavior: fails before export

### `table_cell_uuid_numeric`

- Relevant KiCad source:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:4898`
- Internal parser path:
  `table_cell` `uuid` uses `NeedSYMBOL` plus `parseKIID`, so bare numeric tokens are rejected
- Observation:
  a `table_cell` with a numeric `uuid` fails through `kicad-cli` and surfaces only the generic
  message `Failed to load schematic`
- Severity: error
- Export behavior: fails before export

### `table_cell_uuid_list`

- Relevant KiCad source:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:4898`
- Internal parser path:
  `table_cell` `uuid` requires an atom; list-shaped payloads fail before export
- Observation:
  a `table_cell` with a list-shaped `uuid` fails through `kicad-cli` and surfaces only the generic
  message `Failed to load schematic`
- Severity: error
- Export behavior: fails before export

### `group_uuid_list`

- Relevant KiCad source:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:5173`
- Internal parser path:
  group metadata `uuid` must be atom-shaped; list payloads fail before export
- Observation:
  a `group` with a list-shaped `uuid` fails through `kicad-cli` and surfaces only the generic
  message `Failed to load schematic`
- Severity: error
- Export behavior: fails before export

### `group_uuid_symbol_ok`

- Relevant KiCad source:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:5173`
- Internal parser path:
  `group/uuid` accepts a symbol token and routes it through `parseKIID()`
- Observation:
  `group "G" (uuid foo)` still exports through `kicad-cli` with exit 0 and no visible diagnostics,
  and `ki extract` matches that accepted form
- Severity: none
- Export behavior: succeeds

### `rule_area_uuid_numeric`

- Relevant KiCad source:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:4593`
- Internal parser path:
  `rule_area` `uuid` uses `NeedSYMBOL` plus `parseKIID`, so bare numeric tokens are rejected
- Observation:
  a `rule_area` with a numeric `uuid` fails through `kicad-cli` and surfaces only the generic
  message `Failed to load schematic`
- Severity: error
- Export behavior: fails before export

### `rule_area_uuid_list`

- Relevant KiCad source:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:4593`
- Internal parser path:
  `rule_area` `uuid` requires an atom; list-shaped payloads fail before export
- Observation:
  a `rule_area` with a list-shaped `uuid` fails through `kicad-cli` and surfaces only the generic
  message `Failed to load schematic`
- Severity: error
- Export behavior: fails before export

### `table_uuid_numeric`

- Relevant KiCad source:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:5059`
- Internal parser path:
  `table` `uuid` uses `NeedSYMBOL` plus `parseKIID`, so bare numeric tokens are rejected
- Observation:
  a `table` with a numeric `uuid` fails through `kicad-cli` and surfaces only the generic message
  `Failed to load schematic`
- Severity: error
- Export behavior: fails before export

### `table_uuid_list`

- Relevant KiCad source:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:5059`
- Internal parser path:
  `table` `uuid` requires an atom; list-shaped payloads fail before export
- Observation:
  a `table` with a list-shaped `uuid` fails through `kicad-cli` and surfaces only the generic
  message `Failed to load schematic`
- Severity: error
- Export behavior: fails before export

### `sheet_uuid_numeric`

- Relevant KiCad source:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:3752`
- Internal parser path:
  `sheet` `uuid` uses `NeedSYMBOL` plus `parseKIID`, so bare numeric tokens are rejected
- Observation:
  a `sheet` with a numeric `uuid` fails through `kicad-cli` and surfaces only the generic message
  `Failed to load schematic`
- Severity: error
- Export behavior: fails before export

### `sheet_uuid_list`

- Relevant KiCad source:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:3752`
- Internal parser path:
  `sheet` `uuid` requires an atom; list-shaped payloads fail before export
- Observation:
  a `sheet` with a list-shaped `uuid` fails through `kicad-cli` and surfaces only the generic
  message `Failed to load schematic`
- Severity: error
- Export behavior: fails before export

### `sheet_pin_uuid_numeric`

- Relevant KiCad source:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:2504`
- Internal parser path:
  sheet-pin `uuid` uses `NeedSYMBOL` plus `parseKIID`, so bare numeric tokens are rejected
- Observation:
  a sheet pin with a numeric `uuid` fails through `kicad-cli` and surfaces only the generic
  message `Failed to load schematic`
- Severity: error
- Export behavior: fails before export

### `sheet_pin_uuid_list`

- Relevant KiCad source:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:2504`
- Internal parser path:
  sheet-pin `uuid` requires an atom; list-shaped payloads fail before export
- Observation:
  a sheet pin with a list-shaped `uuid` fails through `kicad-cli` and surfaces only the generic
  message `Failed to load schematic`
- Severity: error
- Export behavior: fails before export

### `sheet_pin_with_href`

- Relevant KiCad source:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:2510`
- Internal parser path:
  sheet pins do not accept a direct `href` child
- Observation:
  a sheet pin carrying a direct `href` child fails through `kicad-cli` and surfaces only the
  generic message `Failed to load schematic`
- Severity: error
- Export behavior: fails before export

### `sheet_pin_effects_href`

- Relevant KiCad sources:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:768`
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:2501`
- Internal parser path:
  sheet-pin `effects` parsing rejects hyperlink-bearing text
- Observation:
  a sheet pin whose `effects` block contains `href` fails through `kicad-cli` and surfaces only
  the generic message `Failed to load schematic`
- Severity: error
- Export behavior: fails before export

### `label_uuid_numeric`

- Relevant KiCad source:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:4733`
- Internal parser path:
  plain `label` `uuid` uses `NeedSYMBOL` plus `parseKIID`, so bare numeric tokens are rejected
- Observation:
  a `label` with a numeric `uuid` fails through `kicad-cli` and surfaces only the generic message
  `Failed to load schematic`
- Severity: error
- Export behavior: fails before export

### `label_uuid_list`

- Relevant KiCad source:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:4733`
- Internal parser path:
  plain `label` `uuid` requires an atom; list-shaped payloads fail before export
- Observation:
  a `label` with a list-shaped `uuid` fails through `kicad-cli` and surfaces only the generic
  message `Failed to load schematic`
- Severity: error
- Export behavior: fails before export

### `global_label_uuid_numeric`

- Relevant KiCad source:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:4733`
- Internal parser path:
  `global_label` `uuid` uses `NeedSYMBOL` plus `parseKIID`, so bare numeric tokens are rejected
- Observation:
  a `global_label` with a numeric `uuid` fails through `kicad-cli` and surfaces only the generic
  message `Failed to load schematic`
- Severity: error
- Export behavior: fails before export

### `global_label_uuid_list`

- Relevant KiCad source:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:4733`
- Internal parser path:
  `global_label` `uuid` requires an atom; list-shaped payloads fail before export
- Observation:
  a `global_label` with a list-shaped `uuid` fails through `kicad-cli` and surfaces only the
  generic message `Failed to load schematic`
- Severity: error
- Export behavior: fails before export

### `hier_label_uuid_numeric`

- Relevant KiCad source:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:4733`
- Internal parser path:
  `hierarchical_label` `uuid` uses `NeedSYMBOL` plus `parseKIID`, so bare numeric tokens are
  rejected
- Observation:
  a `hierarchical_label` with a numeric `uuid` fails through `kicad-cli` and surfaces only the
  generic message `Failed to load schematic`
- Severity: error
- Export behavior: fails before export

### `hier_label_uuid_list`

- Relevant KiCad source:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:4733`
- Internal parser path:
  `hierarchical_label` `uuid` requires an atom; list-shaped payloads fail before export
- Observation:
  a `hierarchical_label` with a list-shaped `uuid` fails through `kicad-cli` and surfaces only the
  generic message `Failed to load schematic`
- Severity: error
- Export behavior: fails before export

### `directive_label_uuid_numeric`

- Relevant KiCad source:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:4733`
- Internal parser path:
  `directive_label` `uuid` uses `NeedSYMBOL` plus `parseKIID`, so bare numeric tokens are rejected
- Observation:
  a `directive_label` with a numeric `uuid` fails through `kicad-cli` and surfaces only the
  generic message `Failed to load schematic`
- Severity: error
- Export behavior: fails before export

### `directive_label_uuid_list`

- Relevant KiCad source:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:4733`
- Internal parser path:
  `directive_label` `uuid` requires an atom; list-shaped payloads fail before export
- Observation:
  a `directive_label` with a list-shaped `uuid` fails through `kicad-cli` and surfaces only the
  generic message `Failed to load schematic`
- Severity: error
- Export behavior: fails before export

### `label_effects_href`

- Relevant KiCad sources:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:768`
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:4714`
- Internal parser path:
  plain `label` `effects` parsing rejects hyperlink-bearing text
- Observation:
  a `label` whose `effects` block contains `href` fails through `kicad-cli` and surfaces only the
  generic message `Failed to load schematic`
- Severity: error
- Export behavior: fails before export

### `global_label_effects_href`

- Relevant KiCad sources:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:768`
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:4714`
- Internal parser path:
  `global_label` `effects` parsing rejects hyperlink-bearing text
- Observation:
  a `global_label` whose `effects` block contains `href` fails through `kicad-cli` and surfaces
  only the generic message `Failed to load schematic`
- Severity: error
- Export behavior: fails before export

### `hier_label_effects_href`

- Relevant KiCad sources:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:768`
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:4714`
- Internal parser path:
  `hierarchical_label` `effects` parsing rejects hyperlink-bearing text
- Observation:
  a `hierarchical_label` whose `effects` block contains `href` fails through `kicad-cli` and
  surfaces only the generic message `Failed to load schematic`
- Severity: error
- Export behavior: fails before export

### `directive_label_effects_href`

- Relevant KiCad sources:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:768`
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:4714`
- Internal parser path:
  `directive_label` `effects` parsing rejects hyperlink-bearing text
- Observation:
  a `directive_label` whose `effects` block contains `href` fails through `kicad-cli` and surfaces
  only the generic message `Failed to load schematic`
- Severity: error
- Export behavior: fails before export

### `symbol_property_effects_href`

- Relevant KiCad sources:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:2406`
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:768`
- Internal parser path:
  placed-symbol property `effects` parsing rejects hyperlink-bearing text
- Observation:
  a placed symbol property whose `effects` block contains `href` fails through `kicad-cli` and
  surfaces only the generic message `Failed to load schematic`
- Severity: error
- Export behavior: fails before export

### `lib_symbol_property_effects_href`

- Relevant KiCad sources:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:1138`
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:768`
- Internal parser path:
  library-symbol property `effects` parsing rejects hyperlink-bearing text
- Observation:
  a library symbol property whose `effects` block contains `href` fails through `kicad-cli` and
  surfaces only the generic message `Failed to load schematic`
- Severity: error
- Export behavior: fails before export

### `lib_symbol_text_href_direct`

- Relevant KiCad source:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:1941`
- Internal parser path:
  symbol-side `text` only accepts `at` and `effects`; a direct `href` child is invalid
- Observation:
  a library symbol `text` carrying a direct `href` child fails through `kicad-cli` and surfaces
  only the generic message `Failed to load schematic`
- Severity: error
- Export behavior: fails before export

### `lib_symbol_text_effects_href`

- Relevant KiCad sources:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:1985`
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:768`
- Internal parser path:
  symbol-side `text` `effects` parsing rejects hyperlink-bearing text
- Observation:
  a library symbol `text` whose `effects` block contains `href` fails through `kicad-cli` and
  surfaces only the generic message `Failed to load schematic`
- Severity: error
- Export behavior: fails before export

### `lib_pin_name_effects_href`

- Relevant KiCad sources:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:1721`
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:768`
- Internal parser path:
  library-pin name `effects` parsing rejects hyperlink-bearing text
- Observation:
  a library pin `name` whose `effects` block contains `href` fails through `kicad-cli` and
  surfaces only the generic message `Failed to load schematic`
- Severity: error
- Export behavior: fails before export

### `lib_pin_number_effects_href`

- Relevant KiCad sources:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:1761`
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:768`
- Internal parser path:
  library-pin number `effects` parsing rejects hyperlink-bearing text
- Observation:
  a library pin `number` whose `effects` block contains `href` fails through `kicad-cli` and
  surfaces only the generic message `Failed to load schematic`
- Severity: error
- Export behavior: fails before export

### `lib_symbol_textbox_href_direct`

- Relevant KiCad source:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:2001`
- Internal parser path:
  symbol-side `text_box` does not accept a direct `href` child
- Observation:
  a library symbol `text_box` carrying a direct `href` child fails through `kicad-cli` and
  surfaces only the generic message `Failed to load schematic`
- Severity: error
- Export behavior: fails before export

### `sheet_property_effects_href`

- Relevant KiCad sources:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:2406`
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:768`
- Internal parser path:
  sheet property `effects` parsing rejects hyperlink-bearing text
- Observation:
  a sheet property whose `effects` block contains `href` fails through `kicad-cli` and surfaces
  only the generic message `Failed to load schematic`
- Severity: error
- Export behavior: fails before export

### `lib_symbol_textbox_effects_href`

- Relevant KiCad sources:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:2094`
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:768`
- Internal parser path:
  symbol-side `text_box` `effects` parsing rejects hyperlink-bearing text
- Observation:
  a library symbol `text_box` whose `effects` block contains `href` fails through `kicad-cli` and
  surfaces only the generic message `Failed to load schematic`
- Severity: error
- Export behavior: fails before export

### `symbol_property_effects_duplicate_href`

- Relevant KiCad sources:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:2406`
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:768`
- Internal parser path:
  placed-symbol property `effects` parsing also rejects duplicate `href` entries
- Observation:
  a placed symbol property whose `effects` block contains duplicate `href` entries fails through
  `kicad-cli` and surfaces only the generic message `Failed to load schematic`
- Severity: error
- Export behavior: fails before export

### `sheet_property_effects_duplicate_href`

- Relevant KiCad sources:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:2406`
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:768`
- Internal parser path:
  sheet property `effects` parsing also rejects duplicate `href` entries
- Observation:
  a sheet property whose `effects` block contains duplicate `href` entries fails through
  `kicad-cli` and surfaces only the generic message `Failed to load schematic`
- Severity: error
- Export behavior: fails before export

### `lib_symbol_property_effects_duplicate_href`

- Relevant KiCad sources:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:1987`
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:768`
- Internal parser path:
  library-symbol property `effects` parsing also rejects duplicate `href` entries
- Observation:
  a library symbol property whose `effects` block contains duplicate `href` entries fails through
  `kicad-cli` and surfaces only the generic message `Failed to load schematic`
- Severity: error
- Export behavior: fails before export

### `lib_pin_name_effects_duplicate_href`

- Relevant KiCad sources:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:2171`
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:768`
- Internal parser path:
  library pin-name `effects` parsing also rejects duplicate `href` entries
- Observation:
  a library pin name whose `effects` block contains duplicate `href` entries fails through
  `kicad-cli` and surfaces only the generic message `Failed to load schematic`
- Severity: error
- Export behavior: fails before export

### `lib_pin_number_effects_duplicate_href`

- Relevant KiCad sources:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:2190`
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:768`
- Internal parser path:
  library pin-number `effects` parsing also rejects duplicate `href` entries
- Observation:
  a library pin number whose `effects` block contains duplicate `href` entries fails through
  `kicad-cli` and surfaces only the generic message `Failed to load schematic`
- Severity: error
- Export behavior: fails before export

### `lib_symbol_textbox_effects_duplicate_href`

- Relevant KiCad sources:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:2094`
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:768`
- Internal parser path:
  library symbol `text_box` `effects` parsing also rejects duplicate `href` entries
- Observation:
  a library symbol `text_box` whose `effects` block contains duplicate `href` entries fails
  through `kicad-cli` and surfaces only the generic message `Failed to load schematic`
- Severity: error
- Export behavior: fails before export

### `sheet_pin_effects_duplicate_href`

- Relevant KiCad sources:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:1201`
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:768`
- Internal parser path:
  sheet-pin `effects` parsing also rejects duplicate `href` entries
- Observation:
  a sheet pin whose `effects` block contains duplicate `href` entries fails through `kicad-cli`
  and surfaces only the generic message `Failed to load schematic`
- Severity: error
- Export behavior: fails before export

### `text_box_duplicate_fill`

- Relevant KiCad sources:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:4877`
- Internal parser path:
  repeated `fill` children on `text_box` are accepted on the export load path
- Observation:
  a `text_box` with duplicate `fill` children exports through `kicad-cli` with exit 0 and no
  visible diagnostics
- Severity: none
- Export behavior: succeeds

### `text_box_duplicate_span`

- Relevant KiCad sources:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:4858`
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:4866`
- Internal parser path:
  `text_box` rejects table-cell-only `span` content on load, including the repeated-child form
- Observation:
  a `text_box` carrying duplicate `span` children fails through `kicad-cli` and surfaces only the
  generic message `Failed to load schematic`
- Severity: error
- Export behavior: fails before export

### `text_box_duplicate_stroke`

- Relevant KiCad sources:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:4872`
- Internal parser path:
  repeated `stroke` children on `text_box` are accepted by the export load path
- Observation:
  a `text_box` with duplicate `stroke` children exports through `kicad-cli` with exit 0 and no
  visible diagnostics
- Severity: none
- Export behavior: succeeds

### `text_box_duplicate_margins`

- Relevant KiCad sources:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:4884`
- Internal parser path:
  repeated `margins` children on `text_box` are accepted on the export load path
- Observation:
  a `text_box` with duplicate `margins` children exports through `kicad-cli` with exit 0 and no
  visible diagnostics
- Severity: none
- Export behavior: succeeds

### `text_box_duplicate_exclude_from_sim`

- Relevant KiCad sources:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:4830`
- Internal parser path:
  repeated `exclude_from_sim` on `text_box` is accepted by the export load path
- Observation:
  a `text_box` with duplicate `exclude_from_sim` children exports through `kicad-cli` with exit 0
  and no visible diagnostics
- Severity: none
- Export behavior: succeeds

### `table_cell_duplicate_at`

- Relevant KiCad sources:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:4846`
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:4905`
- Internal parser path:
  repeated `at` children on `table_cell` are accepted on the export load path
- Observation:
  a `table_cell` with duplicate `at` children exports through `kicad-cli` with exit 0 and no
  visible diagnostics
- Severity: none
- Export behavior: succeeds

### `table_cell_duplicate_fill`

- Relevant KiCad sources:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:4877`
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:4905`
- Internal parser path:
  repeated `fill` children on `table_cell` are accepted on the export load path
- Observation:
  a `table_cell` with duplicate `fill` children exports through `kicad-cli` with exit 0 and no
  visible diagnostics
- Severity: none
- Export behavior: succeeds

### `table_cell_duplicate_size`

- Relevant KiCad sources:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:4852`
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:4905`
- Internal parser path:
  repeated `size` children on `table_cell` are accepted on the export load path
- Observation:
  a `table_cell` with duplicate `size` children exports through `kicad-cli` with exit 0 and no
  visible diagnostics
- Severity: none
- Export behavior: succeeds

### `table_cell_duplicate_span`

- Relevant KiCad sources:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:4858`
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:4905`
- Internal parser path:
  repeated `span` children on `table_cell` are accepted on the export load path
- Observation:
  a `table_cell` with duplicate `span` children exports through `kicad-cli` with exit 0 and no
  visible diagnostics
- Severity: none
- Export behavior: succeeds

### `table_cell_duplicate_stroke`

- Relevant KiCad sources:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:4872`
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:4905`
- Internal parser path:
  repeated `stroke` children on `table_cell` are accepted on the export load path
- Observation:
  a `table_cell` with duplicate `stroke` children exports through `kicad-cli` with exit 0 and no
  visible diagnostics
- Severity: none
- Export behavior: succeeds

### `table_border_duplicate_external`

- Relevant KiCad sources:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:4993`
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:5003`
- Internal parser path:
  duplicate `external` entries inside a table `border` block are accepted on the export load path
- Observation:
  a table `border` block with duplicate `external` children exports through `kicad-cli` with
  exit 0 and no visible diagnostics
- Severity: none
- Export behavior: succeeds

### `table_border_duplicate_stroke`

- Relevant KiCad sources:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:4993`
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:5013`
- Internal parser path:
  duplicate `stroke` entries inside a table `border` block are accepted on the export load path
- Observation:
  a table `border` block with duplicate `stroke` children exports through `kicad-cli` with exit
  0 and no visible diagnostics
- Severity: none
- Export behavior: succeeds

### `table_duplicate_uuid`

- Relevant KiCad sources:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:5059`
- Internal parser path:
  duplicate top-level `uuid` children on `table` are accepted on the export load path
- Observation:
  a table with duplicate `uuid` children exports through `kicad-cli` with exit 0 and no visible
  diagnostics
- Severity: none
- Export behavior: succeeds

### `table_separators_duplicate_stroke`

- Relevant KiCad sources:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:5026`
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:5046`
- Internal parser path:
  duplicate `stroke` entries inside a table `separators` block are accepted on the export load
  path
- Observation:
  a table `separators` block with duplicate `stroke` children exports through `kicad-cli` with
  exit 0 and no visible diagnostics
- Severity: none
- Export behavior: succeeds

### `invalid_table_cell_at_arity`

- Relevant KiCad sources:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:4846`
- Internal parser path:
  `table_cell` uses the textbox-content parser, and a four-value `at` tuple fails on the current
  table syntax path
- Observation:
  a modern-syntax `table_cell` with a four-value `at` tuple fails through `kicad-cli` and
  surfaces only the generic message `Failed to load schematic`
- Severity: error
- Export behavior: fails before export

### `invalid_table_cell_margins_token`

- Relevant KiCad sources:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:4884`
- Internal parser path:
  malformed `table_cell/margins` content fails on the current table syntax path
- Observation:
  a modern-syntax `table_cell` with a non-numeric trailing `margins` token fails through
  `kicad-cli` and surfaces only the generic message `Failed to load schematic`
- Severity: error
- Export behavior: fails before export

### `invalid_table_cell_size_arity`

- Relevant KiCad sources:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:4852`
- Internal parser path:
  `table_cell/size` still requires a two-value tuple on the current table syntax path
- Observation:
  a modern-syntax `table_cell` with a three-value `size` tuple fails through `kicad-cli` and
  surfaces only the generic message `Failed to load schematic`
- Severity: error
- Export behavior: fails before export

### `invalid_table_cell_span_token`

- Relevant KiCad sources:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:4858`
- Internal parser path:
  malformed `table_cell/span` content fails on the current table syntax path
- Observation:
  a modern-syntax `table_cell` with a non-numeric trailing `span` token fails through `kicad-cli`
  and surfaces only the generic message `Failed to load schematic`
- Severity: error
- Export behavior: fails before export

### `invalid_text_box_at_arity`

- Relevant KiCad sources:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:4846`
- Internal parser path:
  `text_box` rejects a four-value `at` tuple on the current textbox-content parser path
- Observation:
  a `text_box` with a four-value `at` tuple fails through `kicad-cli` and surfaces only the
  generic message `Failed to load schematic`
- Severity: error
- Export behavior: fails before export

### `text_box_duplicate_start`

- Relevant KiCad sources:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:4835`
- Internal parser path:
  the legacy `text_box/start` compatibility branch accepts duplicate `start` children
- Observation:
  a `text_box` with duplicate legacy `start` children exports through `kicad-cli` with exit 0 and
  no visible diagnostics
- Severity: none
- Export behavior: succeeds

### `text_box_start_arity`

- Relevant KiCad sources:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:4835`
- Internal parser path:
  the legacy `text_box/start` compatibility branch still requires a two-value tuple
- Observation:
  a `text_box` with a three-value legacy `start` tuple fails through `kicad-cli` and surfaces
  only the generic message `Failed to load schematic`
- Severity: error
- Export behavior: fails before export

### `text_box_end_arity`

- Relevant KiCad sources:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:4840`
- Internal parser path:
  the legacy `text_box/end` compatibility branch still requires a two-value tuple
- Observation:
  a `text_box` with a three-value legacy `end` tuple fails through `kicad-cli` and surfaces only
  the generic message `Failed to load schematic`
- Severity: error
- Export behavior: fails before export

### `table_cell_duplicate_start`

- Relevant KiCad sources:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:4835`
- Internal parser path:
  the shared legacy `start` compatibility branch tolerates duplicate `start` children inside
  modern `table_cell` content
- Observation:
  a modern-syntax `table_cell` with duplicate legacy `start` children exports through `kicad-cli`
  with exit 0 and no visible diagnostics
- Severity: none
- Export behavior: succeeds

### `table_cell_start_arity`

- Relevant KiCad sources:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:4835`
- Internal parser path:
  the shared legacy `start` compatibility branch still requires a two-value tuple inside
  `table_cell`
- Observation:
  a modern-syntax `table_cell` with a three-value legacy `start` tuple fails through `kicad-cli`
  and surfaces only the generic message `Failed to load schematic`
- Severity: error
- Export behavior: fails before export

### `table_cell_end_arity`

- Relevant KiCad sources:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:4840`
- Internal parser path:
  the shared legacy `end` compatibility branch still requires a two-value tuple inside
  `table_cell`
- Observation:
  a modern-syntax `table_cell` with a three-value legacy `end` tuple fails through `kicad-cli`
  and surfaces only the generic message `Failed to load schematic`
- Severity: error
- Export behavior: fails before export

### `global_label_iref_arity`

- Relevant KiCad sources:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:4720`
- Internal parser path:
  the legacy global-label `iref` branch still requires a two-value position tuple
- Observation:
  a global label with a three-value legacy `iref` tuple fails through `kicad-cli` and surfaces
  only the generic message `Failed to load schematic`
- Severity: error
- Export behavior: fails before export

### `global_label_iref_token`

- Relevant KiCad sources:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:4720`
- Internal parser path:
  the legacy global-label `iref` branch rejects non-numeric position tokens
- Observation:
  a global label with a malformed legacy `iref` token fails through `kicad-cli` and surfaces only
  the generic message `Failed to load schematic`
- Severity: error
- Export behavior: fails before export

### `global_label_duplicate_intersheetrefs`

- Relevant KiCad sources:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:4744`
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:4747`
- Internal parser path:
  duplicate mandatory global-label `Intersheetrefs` properties are accepted on the special
  replacement branch that copies mandatory fields into the existing label field slot
- Observation:
  a global label with duplicate mandatory `Intersheetrefs` properties exports through `kicad-cli`
  with exit 0 and no visible diagnostics
- Severity: none
- Export behavior: succeeds

### `invalid_image_at_arity`

- Relevant KiCad sources:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:3611`
- Internal parser path:
  `image/at` still requires a two-value tuple on the current image parser path
- Observation:
  an image with a three-value `at` tuple fails through `kicad-cli` and surfaces only the generic
  message `Failed to load schematic`
- Severity: error
- Export behavior: fails before export

### `invalid_image_scale_token`

- Relevant KiCad sources:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:3616`
- Internal parser path:
  `image/scale` rejects non-numeric tokens on the current image parser path
- Observation:
  an image with a non-numeric `scale` token fails through `kicad-cli` and surfaces only the
  generic message `Failed to load schematic`
- Severity: error
- Export behavior: fails before export

### `image_scale_quoted_numeric`

- Relevant KiCad sources:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:3616`
- Internal parser path:
  `image/scale` rejects quoted numerics on the current image parser path
- Observation:
  an image with `scale "2"` fails through `kicad-cli` and surfaces only the generic message
  `Failed to load schematic`
- Severity: error
- Export behavior: fails before export

### `global_legacy_intersheetrefs_duplicate`

- Relevant KiCad sources:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:2370`
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:4744`
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:4747`
- Internal parser path:
  the legacy global-label field-name alias `Intersheet References` maps into the same mandatory
  replacement branch, and duplicate mandatory intersheet-ref properties are accepted on that path
- Observation:
  a global label carrying both `Intersheetrefs` and legacy `Intersheet References` mandatory
  properties exports through `kicad-cli` with exit 0 and no visible diagnostics
- Severity: none
- Export behavior: succeeds

### `sheet_legacy_sheet_name_at_arity`

- Relevant KiCad sources:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:2353`
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:2394`
- Internal parser path:
  the legacy sheet mandatory field-name alias `Sheet name` still parses through the shared field
  loader, and malformed `at` arity fails there
- Observation:
  a sheet using legacy `Sheet name` with a four-value `at` tuple fails through `kicad-cli` and
  surfaces only the generic message `Failed to load schematic`
- Severity: error
- Export behavior: fails before export

### `global_legacy_intersheetrefs_hide_token`

- Relevant KiCad sources:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:2370`
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:2400`
- Internal parser path:
  the legacy global-label `Intersheet References` alias still parses through the field loader, and
  malformed `hide` tokens fail on that branch
- Observation:
  a global label using legacy `Intersheet References` with `hide maybe` fails through `kicad-cli`
  and surfaces only the generic message `Failed to load schematic`
- Severity: error
- Export behavior: fails before export

### `global_legacy_intersheetrefs_show_name_token`

- Relevant KiCad sources:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:2370`
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:2410`
- Internal parser path:
  the legacy global-label `Intersheet References` alias still parses through the field loader, and
  malformed `show_name` tokens fail on that branch
- Observation:
  a global label using legacy `Intersheet References` with `show_name maybe` fails through
  `kicad-cli` and surfaces only the generic message `Failed to load schematic`
- Severity: error
- Export behavior: fails before export

### `global_legacy_intersheetrefs_bare_show_name`

- Relevant KiCad sources:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:2370`
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:2410`
- Internal parser path:
  the legacy global-label `Intersheet References` alias still parses through the field loader, and
  bare `show_name` uses the same `parseMaybeAbsentBool( true )` branch
- Observation:
  a global label using legacy `Intersheet References` with bare `show_name` is accepted through
  `kicad-cli` with no visible diagnostics
- Severity: none
- Export behavior: exports successfully

### `sheet_legacy_sheet_name_hide_token`

- Relevant KiCad sources:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:2353`
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:2400`
- Internal parser path:
  the legacy sheet mandatory field-name alias `Sheet name` still parses through the field loader,
  and malformed `hide` tokens fail there
- Observation:
  a sheet using legacy `Sheet name` with `hide maybe` fails through `kicad-cli` and surfaces only
  the generic message `Failed to load schematic`
- Severity: error
- Export behavior: fails before export

### `sheet_legacy_sheet_name_show_name_token`

- Relevant KiCad sources:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:2353`
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:2410`
- Internal parser path:
  the legacy sheet mandatory field-name alias `Sheet name` still parses through the field loader,
  and malformed `show_name` tokens fail there
- Observation:
  a sheet using legacy `Sheet name` with `show_name maybe` fails through `kicad-cli` and surfaces
  only the generic message `Failed to load schematic`
- Severity: error
- Export behavior: fails before export

### `sheet_legacy_sheet_name_bare_show_name`

- Relevant KiCad sources:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:2353`
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:2410`
- Internal parser path:
  the legacy sheet mandatory field-name alias `Sheet name` still parses through the field loader,
  and bare `show_name` uses the same `parseMaybeAbsentBool( true )` branch
- Observation:
  a sheet using legacy `Sheet name` with bare `show_name` is accepted through `kicad-cli` with no
  visible diagnostics
- Severity: none
- Export behavior: exports successfully

### `global_legacy_intersheetrefs_do_not_autoplace_token`

- Relevant KiCad sources:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:2370`
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:2417`
- Internal parser path:
  the legacy global-label `Intersheet References` alias still parses through the field loader, and
  malformed `do_not_autoplace` tokens fail on that branch
- Observation:
  a global label using legacy `Intersheet References` with `do_not_autoplace maybe` fails through
  `kicad-cli` and surfaces only the generic message `Failed to load schematic`
- Severity: error
- Export behavior: fails before export

### `global_legacy_intersheetrefs_bare_do_not_autoplace`

- Relevant KiCad sources:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:2370`
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:2417`
- Internal parser path:
  the legacy global-label `Intersheet References` alias still parses through the field loader, and
  bare `do_not_autoplace` uses the same `parseMaybeAbsentBool( true )` branch
- Observation:
  a global label using legacy `Intersheet References` with bare `do_not_autoplace` is accepted
  through `kicad-cli` with no visible diagnostics
- Severity: none
- Export behavior: exports successfully

### `sheet_legacy_sheet_file_hide_token`

- Relevant KiCad sources:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:2355`
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:2400`
- Internal parser path:
  the legacy sheet mandatory field-name alias `Sheet file` still parses through the field loader,
  and malformed `hide` tokens fail there
- Observation:
  a sheet using legacy `Sheet file` with `hide maybe` fails through `kicad-cli` and surfaces only
  the generic message `Failed to load schematic`
- Severity: error
- Export behavior: fails before export

### `sheet_legacy_sheet_file_show_name_token`

- Relevant KiCad sources:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:2355`
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:2410`
- Internal parser path:
  the legacy sheet mandatory field-name alias `Sheet file` still parses through the field loader,
  and malformed `show_name` tokens fail there
- Observation:
  a sheet using legacy `Sheet file` with `show_name maybe` fails through `kicad-cli` and surfaces
  only the generic message `Failed to load schematic`
- Severity: error
- Export behavior: fails before export

### `sheet_legacy_sheet_file_bare_show_name`

- Relevant KiCad sources:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:2355`
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:2410`
- Internal parser path:
  the legacy sheet mandatory field-name alias `Sheet file` still parses through the field loader,
  and bare `show_name` uses the same `parseMaybeAbsentBool( true )` branch
- Observation:
  a sheet using legacy `Sheet file` with bare `show_name` is accepted through `kicad-cli` with no
  visible diagnostics
- Severity: none
- Export behavior: exports successfully

### `sheet_legacy_sheet_name_do_not_autoplace_token`

- Relevant KiCad sources:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:2353`
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:2417`
- Internal parser path:
  the legacy sheet mandatory field-name alias `Sheet name` still parses through the field loader,
  and malformed `do_not_autoplace` tokens fail there
- Observation:
  a sheet using legacy `Sheet name` with `do_not_autoplace maybe` fails through `kicad-cli` and
  surfaces only the generic message `Failed to load schematic`
- Severity: error
- Export behavior: fails before export

### `sheet_legacy_sheet_name_bare_do_not_autoplace`

- Relevant KiCad sources:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:2353`
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:2417`
- Internal parser path:
  the legacy sheet mandatory field-name alias `Sheet name` still parses through the field loader,
  and bare `do_not_autoplace` uses the same `parseMaybeAbsentBool( true )` branch
- Observation:
  a sheet using legacy `Sheet name` with bare `do_not_autoplace` is accepted through `kicad-cli`
  with no visible diagnostics
- Severity: none
- Export behavior: exports successfully

### `global_legacy_intersheetrefs_id_token`

- Relevant KiCad sources:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:2370`
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:2389`
- Internal parser path:
  the legacy global-label `Intersheet References` alias still parses through the field loader, and
  malformed legacy `id` payloads fail on that branch
- Observation:
  a global label using legacy `Intersheet References` with `id nope` fails through `kicad-cli`
  and surfaces only the generic message `Failed to load schematic`
- Severity: error
- Export behavior: fails before export

### `sheet_legacy_sheet_file_do_not_autoplace_token`

- Relevant KiCad sources:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:2355`
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:2417`
- Internal parser path:
  the legacy sheet mandatory field-name alias `Sheet file` still parses through the field loader,
  and malformed `do_not_autoplace` tokens fail there
- Observation:
  a sheet using legacy `Sheet file` with `do_not_autoplace maybe` fails through `kicad-cli` and
  surfaces only the generic message `Failed to load schematic`
- Severity: error
- Export behavior: fails before export

### `sheet_legacy_sheet_file_bare_do_not_autoplace`

- Relevant KiCad sources:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:2355`
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:2417`
- Internal parser path:
  the legacy sheet mandatory field-name alias `Sheet file` still parses through the field loader,
  and bare `do_not_autoplace` uses the same `parseMaybeAbsentBool( true )` branch
- Observation:
  a sheet using legacy `Sheet file` with bare `do_not_autoplace` is accepted through `kicad-cli`
  with no visible diagnostics
- Severity: none
- Export behavior: exports successfully

### `sheet_legacy_sheet_file_id_token`

- Relevant KiCad sources:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:2355`
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:2389`
- Internal parser path:
  the legacy sheet mandatory field-name alias `Sheet file` still parses through the field loader,
  and malformed legacy `id` payloads fail there
- Observation:
  a sheet using legacy `Sheet file` with `id nope` fails through `kicad-cli` and surfaces only
  the generic message `Failed to load schematic`
- Severity: error
- Export behavior: fails before export

### `bus_alias_empty_members`

- Relevant KiCad sources:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:5108`
- Internal parser path:
  a `bus_alias` with an empty `members` list is accepted on the current alias parser path
- Observation:
  a `bus_alias` with empty `members` exports through `kicad-cli` with exit 0 and no visible
  diagnostics
- Severity: none
- Export behavior: succeeds

### `bus_alias_duplicate_member`

- Relevant KiCad sources:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:5108`
- Internal parser path:
  duplicate `members` entries on `bus_alias` are accepted on the current alias parser path
- Observation:
  a `bus_alias` with duplicate members exports through `kicad-cli` with exit 0 and no visible
  diagnostics
- Severity: none
- Export behavior: succeeds

### `bus_alias_extra_child`

- Relevant KiCad sources:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:5123`
- Internal parser path:
  a trailing extra child after `bus_alias/members` fails on the current alias parser path
- Observation:
  a `bus_alias` with an extra trailing child fails through `kicad-cli` and surfaces only the
  generic message `Failed to load schematic`
- Severity: error
- Export behavior: fails before export

### `group_duplicate_member`

- Relevant KiCad sources:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:5129`
- Internal parser path:
  duplicate member UUIDs inside a `group/members` list are accepted on the current group parser
  path
- Observation:
  a `group` with duplicate member UUIDs exports through `kicad-cli` with exit 0 and no visible
  diagnostics
- Severity: none
- Export behavior: succeeds

### `group_empty_members`

- Relevant KiCad sources:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:5129`
- Internal parser path:
  an empty `group/members` list is accepted on the current group parser path
- Observation:
  a `group` with empty members exports through `kicad-cli` with exit 0 and no visible
  diagnostics
- Severity: none
- Export behavior: succeeds

### `group_missing_member_uuid`

- Relevant KiCad sources:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:5129`
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:5266`
- Internal parser path:
  unresolved member UUIDs are accepted during the current schematic group parse/load path used by
  `kicad-cli sch export netlist`
- Observation:
  a `group` whose members reference a missing UUID exports through `kicad-cli` with exit 0 and no
  visible diagnostics
- Severity: none
- Export behavior: succeeds

### `bus_entry_extra_child`

- Relevant KiCad sources:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:4124`
- Internal parser path:
  `bus_entry` rejects trailing unexpected children after its known fields
- Observation:
  a `bus_entry` with an extra trailing child fails through `kicad-cli` and surfaces only the
  generic message `Failed to load schematic`
- Severity: error
- Export behavior: fails before export

### `arc_extra_child`

- Relevant KiCad sources:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:4333`
- Internal parser path:
  `arc` rejects trailing unexpected children after its known geometry/styling fields
- Observation:
  an `arc` with an extra trailing child fails through `kicad-cli` and surfaces only the generic
  message `Failed to load schematic`
- Severity: error
- Export behavior: fails before export

### `bezier_extra_child`

- Relevant KiCad sources:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:4598`
- Internal parser path:
  `bezier` rejects trailing unexpected children after its known geometry/styling fields
- Observation:
  a `bezier` with an extra trailing child fails through `kicad-cli` and surfaces only the generic
  message `Failed to load schematic`
- Severity: error
- Export behavior: fails before export

### `bus_extra_child`

- Relevant KiCad sources:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:4269`
- Internal parser path:
  `bus` rejects trailing unexpected children after its known fields
- Observation:
  a `bus` with an extra trailing child fails through `kicad-cli` and surfaces only the generic
  message `Failed to load schematic`
- Severity: error
- Export behavior: fails before export

### `circle_extra_child`

- Relevant KiCad sources:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:4393`
- Internal parser path:
  `circle` rejects trailing unexpected children after its known geometry/styling fields
- Observation:
  a `circle` with an extra trailing child fails through `kicad-cli` and surfaces only the generic
  message `Failed to load schematic`
- Severity: error
- Export behavior: fails before export

### `polyline_extra_child`

- Relevant KiCad sources:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:4200`
- Internal parser path:
  `polyline` rejects trailing unexpected children after its known geometry/styling fields
- Observation:
  a `polyline` with an extra trailing child fails through `kicad-cli` and surfaces only the
  generic message `Failed to load schematic`
- Severity: error
- Export behavior: fails before export

### `rect_extra_child`

- Relevant KiCad sources:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:4457`
- Internal parser path:
  `rectangle` rejects trailing unexpected children after its known geometry/styling fields
- Observation:
  a `rectangle` with an extra trailing child fails through `kicad-cli` and surfaces only the
  generic message `Failed to load schematic`
- Severity: error
- Export behavior: fails before export

### `wire_extra_child`

- Relevant KiCad sources:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:4269`
- Internal parser path:
  `wire` rejects trailing unexpected children after its known fields
- Observation:
  a `wire` with an extra trailing child fails through `kicad-cli` and surfaces only the generic
  message `Failed to load schematic`
- Severity: error
- Export behavior: fails before export

### `junction_extra_child`

- Relevant KiCad sources:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:4033`
- Internal parser path:
  `junction` rejects trailing unexpected children after its known fields
- Observation:
  a `junction` with an extra trailing child fails through `kicad-cli` and surfaces only the
  generic message `Failed to load schematic`
- Severity: error
- Export behavior: fails before export

### `no_connect_extra_child`

- Relevant KiCad sources:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:4070`
- Internal parser path:
  `no_connect` rejects trailing unexpected children after its known fields
- Observation:
  a `no_connect` with an extra trailing child fails through `kicad-cli` and surfaces only the
  generic message `Failed to load schematic`
- Severity: error
- Export behavior: fails before export

### `image_extra_child`

- Relevant KiCad sources:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:3657`
- Internal parser path:
  `image` rejects trailing unexpected children after its known fields
- Observation:
  an `image` with an extra trailing child fails through `kicad-cli` and surfaces only the generic
  message `Failed to load schematic`
- Severity: error
- Export behavior: fails before export

### `sheet_extra_child`

- Relevant KiCad sources:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:3964`
- Internal parser path:
  `sheet` rejects trailing unexpected children after its known geometry/metadata fields
- Observation:
  a `sheet` with an extra trailing child fails through `kicad-cli` and surfaces only the generic
  message `Failed to load schematic`
- Severity: error
- Export behavior: fails before export

### `symbol_extra_child`

- Relevant KiCad sources:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:3578`
- Internal parser path:
  placed `symbol` rejects trailing unexpected children after its known fields
- Observation:
  a placed `symbol` with an extra trailing child fails through `kicad-cli` and surfaces only the
  generic message `Failed to load schematic`
- Severity: error
- Export behavior: fails before export

### `sheet_pin_extra_child`

- Relevant KiCad sources:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:2511`
- Internal parser path:
  `sheet/pin` rejects trailing unexpected children after `at`, `effects`, and `uuid`
- Observation:
  a sheet `pin` with an extra trailing child fails through `kicad-cli` and surfaces only the
  generic message `Failed to load schematic`
- Severity: error
- Export behavior: fails before export

### `rule_area_extra_child`

- Relevant KiCad sources:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:4525`
- Internal parser path:
  `rule_area` rejects trailing unexpected children after its known fields
- Observation:
  a `rule_area` with an extra trailing child fails through `kicad-cli` and surfaces only the
  generic message `Failed to load schematic`
- Severity: error
- Export behavior: fails before export

### `text_extra_child`

- Relevant KiCad sources:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:4762`
- Internal parser path:
  plain `text` rejects trailing unexpected children after its known fields
- Observation:
  a `text` item with an extra trailing child fails through `kicad-cli` and surfaces only the
  generic message `Failed to load schematic`
- Severity: error
- Export behavior: fails before export

### `label_extra_child`

- Relevant KiCad sources:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:4762`
- Internal parser path:
  local `label` rejects trailing unexpected children after its known fields
- Observation:
  a `label` with an extra trailing child fails through `kicad-cli` and surfaces only the generic
  message `Failed to load schematic`
- Severity: error
- Export behavior: fails before export

### `global_label_extra_child`

- Relevant KiCad sources:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:4762`
- Internal parser path:
  `global_label` rejects trailing unexpected children after its known fields
- Observation:
  a `global_label` with an extra trailing child fails through `kicad-cli` and surfaces only the
  generic message `Failed to load schematic`
- Severity: error
- Export behavior: fails before export

### `hier_label_extra_child`

- Relevant KiCad sources:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:4762`
- Internal parser path:
  `hierarchical_label` rejects trailing unexpected children after its known fields
- Observation:
  a `hierarchical_label` with an extra trailing child fails through `kicad-cli` and surfaces only
  the generic message `Failed to load schematic`
- Severity: error
- Export behavior: fails before export

### `directive_label_extra_child`

- Relevant KiCad sources:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:4762`
- Internal parser path:
  `directive_label` rejects trailing unexpected children after its known fields
- Observation:
  a `directive_label` with an extra trailing child fails through `kicad-cli` and surfaces only the
  generic message `Failed to load schematic`
- Severity: error
- Export behavior: fails before export

### `text_box_extra_child`

- Relevant KiCad sources:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:4908`
- Internal parser path:
  schematic `text_box` rejects trailing unexpected children after its known fields
- Observation:
  a schematic `text_box` with an extra trailing child fails through `kicad-cli` and surfaces only
  the generic message `Failed to load schematic`
- Severity: error
- Export behavior: fails before export

### `table_extra_child`

- Relevant KiCad sources:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:5066`
- Internal parser path:
  `table` rejects trailing unexpected children after its known fields
- Observation:
  a `table` with an extra trailing child fails through `kicad-cli` and surfaces only the generic
  message `Failed to load schematic`
- Severity: error
- Export behavior: fails before export

### `title_block_extra_child`

- Relevant KiCad sources:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:2272`
- Internal parser path:
  `title_block` rejects unexpected children outside the known title metadata entries
- Observation:
  a `title_block` with an extra trailing child fails through `kicad-cli` and surfaces only the
  generic message `Failed to load schematic`
- Severity: error
- Export behavior: fails before export

### `paper_extra_child`

- Relevant KiCad sources:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:2171`
- Internal parser path:
  `paper` rejects extra nested children after the page-type payload
- Observation:
  a `paper` node with an extra nested child fails through `kicad-cli` and surfaces only the
  generic message `Failed to load schematic`
- Severity: error
- Export behavior: fails before export

### `sheet_instances_root_extra_child`

- Relevant KiCad sources:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:2610`
- Internal parser path:
  top-level `sheet_instances` rejects unexpected children outside `path`
- Observation:
  a `sheet_instances` block with an extra trailing child fails through `kicad-cli` and surfaces
  only the generic message `Failed to load schematic`
- Severity: error
- Export behavior: fails before export

### `sheet_instances_path_extra_child`

- Relevant KiCad sources:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:2589`
- Internal parser path:
  `sheet_instances/path` rejects unexpected children outside `page`
- Observation:
  a `sheet_instances/path` with an extra trailing child fails through `kicad-cli` and surfaces
  only the generic message `Failed to load schematic`
- Severity: error
- Export behavior: fails before export

### `symbol_instances_root_extra_child`

- Relevant KiCad sources:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:2697`
- Internal parser path:
  top-level `symbol_instances` rejects unexpected children outside `path`
- Observation:
  a `symbol_instances` block with an extra trailing child fails through `kicad-cli` and surfaces
  only the generic message `Failed to load schematic`
- Severity: error
- Export behavior: fails before export

### `symbol_instances_path_extra_child`

- Relevant KiCad sources:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:2688`
- Internal parser path:
  `symbol_instances/path` rejects unexpected children outside `reference`, `unit`, `value`, and
  `footprint`
- Observation:
  a `symbol_instances/path` with an extra trailing child fails through `kicad-cli` and surfaces
  only the generic message `Failed to load schematic`
- Severity: error
- Export behavior: fails before export

### `nested_instance_project_extra_child`

- Relevant KiCad sources:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:3330`
- Internal parser path:
  placed-symbol `instances/project` rejects unexpected children outside `path`
- Observation:
  a placed-symbol `instances/project` block with an extra trailing child fails through `kicad-cli`
  and surfaces only the generic message `Failed to load schematic`
- Severity: error
- Export behavior: fails before export

### `nested_instance_path_extra_child`

- Relevant KiCad sources:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:3480`
- Internal parser path:
  placed-symbol `instances/project/path` rejects unexpected children outside its known fields
- Observation:
  a placed-symbol `instances/project/path` with an extra trailing child fails through `kicad-cli`
  and surfaces only the generic message `Failed to load schematic`
- Severity: error
- Export behavior: fails before export

### `embedded_fonts_extra_child`

- Relevant KiCad sources:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:3026`
- Internal parser path:
  top-level `embedded_fonts` rejects an extra nested child after the boolean payload
- Observation:
  an `embedded_fonts` node with an extra nested child fails through `kicad-cli` and surfaces only
  the generic message `Failed to load schematic`
- Severity: error
- Export behavior: fails before export

### `netclass_flag_no_visible_diagnostics`

- Relevant KiCad sources:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:4620`
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:4698`
- Internal parser path:
  top-level `netclass_flag` is parsed on the directive-label path and accepts the same
  `length`/`shape`/`fields_autoplaced` structure as KiCad-authored schematics
- Observation:
  a valid `netclass_flag` exports through `kicad-cli` with exit code `0` and no visible
  diagnostics
- Severity: none
- Export behavior: succeeds

### `netclass_flag_extra_child`

- Relevant KiCad sources:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:4762`
- Internal parser path:
  `netclass_flag` rejects trailing unexpected children after its known directive-label fields
- Observation:
  a `netclass_flag` with an extra trailing child fails through `kicad-cli` and surfaces only the
  generic message `Failed to load schematic`
- Severity: error
- Export behavior: fails before export

### `paper_landscape_token`

- Relevant KiCad sources:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:2171`
- Internal parser path:
  `paper` only accepts the optional orientation token `portrait`; other tokens are rejected
- Observation:
  `paper "A4" landscape` fails through `kicad-cli` and surfaces only the generic message
  `Failed to load schematic`
- Severity: error
- Export behavior: fails before export

### `text_with_property`

- Relevant KiCad sources:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:4742`
- Internal parser path:
  plain `text` rejects `property` because text properties are only valid on label-like objects
- Observation:
  a `text` item carrying a `property` child fails through `kicad-cli` and surfaces only the
  generic message `Failed to load schematic`
- Severity: error
- Export behavior: fails before export

### `text_box_with_property`

- Relevant KiCad sources:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:4908`
- Internal parser path:
  schematic `text_box` rejects `property` because the textbox parser only accepts its known
  geometry/style fields
- Observation:
  a schematic `text_box` carrying a `property` child fails through `kicad-cli` and surfaces only
  the generic message `Failed to load schematic`
- Severity: error
- Export behavior: fails before export

### `label_with_iref`

- Relevant KiCad sources:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:4720`
- Internal parser path:
  legacy `iref` is only accepted on `global_label`; local `label` rejects it
- Observation:
  a local `label` carrying an `iref` child fails through `kicad-cli` and surfaces only the generic
  message `Failed to load schematic`
- Severity: error
- Export behavior: fails before export

### `label_with_property`

- Relevant KiCad sources:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:4742`
- Internal parser path:
  local `label` accepts a generic `property` child on the current CLI export path
- Observation:
  a local `label` carrying a `property` child exports successfully through `kicad-cli` with no
  visible diagnostics
- Severity: none
- Export behavior: exports successfully

### `global_label_with_property`

- Relevant KiCad sources:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:4742`
- Internal parser path:
  `global_label` accepts a generic `property` child on the current CLI export path
- Observation:
  a `global_label` carrying a generic `property` child exports successfully through `kicad-cli`
  with no visible diagnostics
- Severity: none
- Export behavior: exports successfully

### `hier_label_with_property`

- Relevant KiCad sources:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:4742`
- Internal parser path:
  `hierarchical_label` accepts a generic `property` child on the current CLI export path
- Observation:
  a `hierarchical_label` carrying a generic `property` child exports successfully through
  `kicad-cli` with no visible diagnostics
- Severity: none
- Export behavior: exports successfully

### `directive_label_with_property`

- Relevant KiCad sources:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:4742`
- Internal parser path:
  plain `directive_label` accepts a generic `property` child on the current CLI export path
- Observation:
  a `directive_label` carrying a generic `property` child exports successfully through `kicad-cli`
  with no visible diagnostics
- Severity: none
- Export behavior: exports successfully

### `label_property_name_list_child`

- Relevant KiCad sources:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:2295`
- Internal parser path:
  `parseSchField()` rejects non-atom property names before label-specific handling
- Observation:
  a local `label` whose `property` name is a nested list fails through `kicad-cli` and surfaces
  only the generic message `Failed to load schematic`
- Severity: error
- Export behavior: fails before export

### `label_property_value_list_child`

- Relevant KiCad sources:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:2311`
- Internal parser path:
  `parseSchField()` rejects non-atom property values before label-specific handling
- Observation:
  a local `label` whose `property` value is a nested list fails through `kicad-cli` and surfaces
  only the generic message `Failed to load schematic`
- Severity: error
- Export behavior: fails before export

### `global_label_property_name_list_child`

- Relevant KiCad sources:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:2295`
- Internal parser path:
  `parseSchField()` rejects non-atom property names on the global-label property path
- Observation:
  a `global_label` whose `property` name is a nested list fails through `kicad-cli` and surfaces
  only the generic message `Failed to load schematic`
- Severity: error
- Export behavior: fails before export

### `global_label_property_value_list_child`

- Relevant KiCad sources:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:2311`
- Internal parser path:
  `parseSchField()` rejects non-atom property values on the global-label property path
- Observation:
  a `global_label` whose `property` value is a nested list fails through `kicad-cli` and surfaces
  only the generic message `Failed to load schematic`
- Severity: error
- Export behavior: fails before export

### `hier_label_property_name_list_child`

- Relevant KiCad sources:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:2295`
- Internal parser path:
  `parseSchField()` rejects non-atom property names on the hierarchical-label property path
- Observation:
  a `hierarchical_label` whose `property` name is a nested list fails through `kicad-cli` and
  surfaces only the generic message `Failed to load schematic`
- Severity: error
- Export behavior: fails before export

### `hier_label_property_value_list_child`

- Relevant KiCad sources:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:2311`
- Internal parser path:
  `parseSchField()` rejects non-atom property values on the hierarchical-label property path
- Observation:
  a `hierarchical_label` whose `property` value is a nested list fails through `kicad-cli` and
  surfaces only the generic message `Failed to load schematic`
- Severity: error
- Export behavior: fails before export

### `directive_label_property_name_list_child`

- Relevant KiCad sources:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:2295`
- Internal parser path:
  `parseSchField()` rejects non-atom property names on the directive-label property path
- Observation:
  a `directive_label` whose `property` name is a nested list fails through `kicad-cli` and
  surfaces only the generic message `Failed to load schematic`
- Severity: error
- Export behavior: fails before export

### `directive_label_property_value_list_child`

- Relevant KiCad sources:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:2311`
- Internal parser path:
  `parseSchField()` rejects non-atom property values on the directive-label property path
- Observation:
  a `directive_label` whose `property` value is a nested list fails through `kicad-cli` and
  surfaces only the generic message `Failed to load schematic`
- Severity: error
- Export behavior: fails before export

### `label_with_length`

- Relevant KiCad sources:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:4698`
- Internal parser path:
  plain `label` rejects the directive-only `length` child
- Observation:
  a local `label` carrying a `length` child fails through `kicad-cli` and surfaces only the
  generic message `Failed to load schematic`
- Severity: error
- Export behavior: fails before export

### `directive_label_with_iref`

- Relevant KiCad sources:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:4762`
- Internal parser path:
  `directive_label` rejects the legacy global-label-only `iref` child
- Observation:
  a `directive_label` carrying `iref` fails through `kicad-cli` and surfaces only the generic
  message `Failed to load schematic`
- Severity: error
- Export behavior: fails before export

### `hier_label_with_iref`

- Relevant KiCad sources:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:4762`
- Internal parser path:
  `hierarchical_label` rejects the legacy global-label-only `iref` child
- Observation:
  a `hierarchical_label` carrying `iref` fails through `kicad-cli` and surfaces only the generic
  message `Failed to load schematic`
- Severity: error
- Export behavior: fails before export

### `netclass_flag_with_iref`

- Relevant KiCad sources:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:4762`
- Internal parser path:
  `netclass_flag` follows the directive-label parser branch and also rejects legacy `iref`
- Observation:
  a `netclass_flag` carrying `iref` fails through `kicad-cli` and surfaces only the generic
  message `Failed to load schematic`
- Severity: error
- Export behavior: fails before export

### `netclass_flag_with_href`

- Relevant KiCad sources:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:4762`
- Internal parser path:
  `netclass_flag` rejects direct `href` children
- Observation:
  a `netclass_flag` carrying direct `href` fails through `kicad-cli` and surfaces only the generic
  message `Failed to load schematic`
- Severity: error
- Export behavior: fails before export

### `netclass_flag_effects_href`

- Relevant KiCad sources:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:4714`
- Internal parser path:
  `netclass_flag` `effects` parsing rejects `href`
- Observation:
  a `netclass_flag` whose `effects` block contains `href` fails through `kicad-cli` and surfaces
  only the generic message `Failed to load schematic`
- Severity: error
- Export behavior: fails before export

### `netclass_flag_uuid_numeric`

- Relevant KiCad sources:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:4733`
- Internal parser path:
  `netclass_flag` rejects numeric UUID payloads
- Observation:
  a `netclass_flag` with numeric `uuid` fails through `kicad-cli` and surfaces only the generic
  message `Failed to load schematic`
- Severity: error
- Export behavior: fails before export

### `netclass_flag_uuid_list`

- Relevant KiCad sources:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:4733`
- Internal parser path:
  `netclass_flag` rejects non-atom UUID payloads
- Observation:
  a `netclass_flag` with list-valued `uuid` fails through `kicad-cli` and surfaces only the
  generic message `Failed to load schematic`
- Severity: error
- Export behavior: fails before export

### `netclass_flag_invalid_shape`

- Relevant KiCad sources:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:4687`
- Internal parser path:
  `netclass_flag` rejects invalid directive-label `shape` tokens
- Observation:
  a `netclass_flag` with invalid `shape` fails through `kicad-cli` and surfaces only the generic
  message `Failed to load schematic`
- Severity: error
- Export behavior: fails before export

### `netclass_flag_invalid_length_token`

- Relevant KiCad sources:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:4698`
- Internal parser path:
  `netclass_flag` rejects non-numeric `length`
- Observation:
  a `netclass_flag` with malformed `length` fails through `kicad-cli` and surfaces only the
  generic message `Failed to load schematic`
- Severity: error
- Export behavior: fails before export

### `netclass_flag_fields_autoplaced_token`

- Relevant KiCad sources:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:4707`
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:164`
- Internal parser path:
  `netclass_flag` rejects malformed `fields_autoplaced` boolean payloads
- Observation:
  a `netclass_flag` with malformed `fields_autoplaced` fails through `kicad-cli` and surfaces only
  the generic message `Failed to load schematic`
- Severity: error
- Export behavior: fails before export

### `netclass_flag_exclude_from_sim_token`

- Relevant KiCad sources:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:4642`
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:138`
- Internal parser path:
  `netclass_flag` rejects malformed `exclude_from_sim` boolean payloads
- Observation:
  a `netclass_flag` with malformed `exclude_from_sim` fails through `kicad-cli` and surfaces only
  the generic message `Failed to load schematic`
- Severity: error
- Export behavior: fails before export

### `bezier_too_many_points`

- Relevant KiCad sources:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:4572`
- Internal parser path:
  top-level schematic `bezier` rejects a fifth control point
- Observation:
  a schematic `bezier` with too many control points fails through `kicad-cli` and surfaces only
  the generic message `Failed to load schematic`
- Severity: error
- Export behavior: fails before export

### `bezier_pts_only`

- Relevant KiCad sources:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:4552`
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:4603`
- Internal parser path:
  top-level schematic `bezier` accepts a `pts` block without requiring explicit `stroke`, `fill`,
  or `uuid`
- Observation:
  a top-level `bezier` with only `pts` exports through `kicad-cli` with exit 0 and no visible
  diagnostics, and `ki extract` matches that accepted form
- Severity: none
- Export behavior: succeeds

### `symbol_bezier_too_many_points`

- Relevant KiCad sources:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:1488`
- Internal parser path:
  symbol-graphics `bezier` also rejects a fifth control point
- Observation:
  a library-symbol `bezier` with too many control points fails through `kicad-cli` and surfaces
  only the generic message `Failed to load schematic`
- Severity: error
- Export behavior: fails before export

### `wire_too_many_points`

- Relevant KiCad sources:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:4243`
- Internal parser path:
  `wire` rejects a third point in its `pts` block
- Observation:
  a `wire` with too many points fails through `kicad-cli` and surfaces only the generic message
  `Failed to load schematic`
- Severity: error
- Export behavior: fails before export

### `bus_too_many_points`

- Relevant KiCad sources:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:4243`
- Internal parser path:
  `bus` rejects a third point in its `pts` block
- Observation:
  a `bus` with too many points fails through `kicad-cli` and surfaces only the generic message
  `Failed to load schematic`
- Severity: error
- Export behavior: fails before export

### `polyline_empty_pts`

- Relevant KiCad sources:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:4194`
- Internal parser path:
  top-level schematic `polyline` rejects an empty `pts` block
- Observation:
  a `polyline` with empty `pts` fails through `kicad-cli` and surfaces only the generic message
  `Failed to load schematic`
- Severity: error
- Export behavior: fails before export

### `paper_user_missing_height`

- Relevant KiCad sources:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:2143`
- Internal parser path:
  `paper "User"` requires both width and height
- Observation:
  `paper "User" 100` fails through `kicad-cli` and surfaces only the generic message
  `Failed to load schematic`
- Severity: error
- Export behavior: fails before export

### `page_user_missing_height_no_visible_diagnostics`

- Relevant KiCad sources:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:2832`
- Internal parser path:
  top-level `page` is a separate two-token sniffing record and accepts `"User" 100`
- Observation:
  `page "User" 100` exports through `kicad-cli` with exit code `0` and no visible diagnostics
- Severity: none
- Export behavior: succeeds

### `page_user_portrait_token`

- Relevant KiCad sources:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:2832`
- Internal parser path:
  top-level `page` rejects extra orientation tokens because it only accepts two scalar payloads
- Observation:
  `page "User" 100 200 portrait` fails through `kicad-cli` and surfaces only the generic message
  `Failed to load schematic`
- Severity: error
- Export behavior: fails before export

### `embedded_files_file_missing_name`

- Relevant KiCad sources:
  - `/Users/Daniel/Desktop/kicad/common/embedded_files.cpp:427`
- Internal parser path:
  an embedded `file` entry cannot begin with `type`; the embedded-files parser requires `name`
  first
- Observation:
  `(embedded_files (file (type other)))` fails through `kicad-cli` and surfaces only the generic
  message `Failed to load schematic`
- Severity: error
- Export behavior: fails before export

### `embedded_files_file_checksum_before_name`

- Relevant KiCad sources:
  - `/Users/Daniel/Desktop/kicad/common/embedded_files.cpp:427`
- Internal parser path:
  an embedded `file` entry rejects `checksum` before `name`
- Observation:
  `(embedded_files (file (checksum abc) (name foo.txt)))` fails through `kicad-cli` and surfaces
  only the generic message `Failed to load schematic`
- Severity: error
- Export behavior: fails before export

### `embedded_files_file_type_before_name`

- Relevant KiCad sources:
  - `/Users/Daniel/Desktop/kicad/common/embedded_files.cpp:461`
- Internal parser path:
  an embedded `file` entry rejects `type` before `name`
- Observation:
  `(embedded_files (file (type other) (name foo.txt)))` fails through `kicad-cli` and surfaces
  only the generic message `Failed to load schematic`
- Severity: error
- Export behavior: fails before export

### `embedded_files_file_data_before_name`

- Relevant KiCad sources:
  - `/Users/Daniel/Desktop/kicad/common/embedded_files.cpp:440`
- Internal parser path:
  an embedded `file` entry rejects `data` before `name`
- Observation:
  `(embedded_files (file (data |abc|) (name foo.txt)))` fails through `kicad-cli` and surfaces
  only the generic message `Failed to load schematic`
- Severity: error
- Export behavior: fails before export

### `embedded_files_file_data_bare_ok`

- Relevant KiCad sources:
  - `/Users/Daniel/Desktop/kicad/common/embedded_files.cpp:440`
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:3048`
- Internal parser path:
  a bare symbol token in `embedded_files/file/data` is tolerated by the schematic-level
  embedded-files wrapper and does not surface as a visible diagnostic
- Observation:
  `(embedded_files (file (name foo.txt) (data abc)))` still exports through `kicad-cli` with exit
  0 and no visible diagnostics, and `ki extract` matches that accepted form
- Severity: none
- Export behavior: succeeds

### `embedded_files_unknown_child_payload`

- Relevant KiCad sources:
  - `/Users/Daniel/Desktop/kicad/common/embedded_files.cpp:407`
- Internal parser path:
  top-level `embedded_files` tolerates a bare unknown child, but an unknown child with payload
  triggers the embedded-files parser failure path
- Observation:
  `(embedded_files (bogus foo))` fails through `kicad-cli` and surfaces only the generic message
  `Failed to load schematic`
- Severity: error
- Export behavior: fails before export

### `embedded_files_bogus_ok`

- Relevant KiCad sources:
  - `/Users/Daniel/Desktop/kicad/common/embedded_files.cpp:399`
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:3048`
- Internal parser path:
  top-level `embedded_files` parse errors are caught by the schematic parser and downgraded to
  non-visible warnings
- Observation:
  `(embedded_files (bogus))` still exports through `kicad-cli` with exit 0 and no visible
  diagnostics, and `ki extract` matches that accepted form
- Severity: none
- Export behavior: succeeds

### `embedded_files_file_empty_ok`

- Relevant KiCad sources:
  - `/Users/Daniel/Desktop/kicad/common/embedded_files.cpp:399`
  - `/Users/Daniel/Desktop/kicad/common/embedded_files.cpp:434`
- Internal parser path:
  an empty `(file)` block inside top-level `embedded_files` is tolerated and dropped
- Observation:
  `(embedded_files (file))` still exports through `kicad-cli` with exit 0 and no visible
  diagnostics, and `ki extract` matches that accepted form
- Severity: none
- Export behavior: succeeds

### `embedded_files_file_bogus_child_ok`

- Relevant KiCad sources:
  - `/Users/Daniel/Desktop/kicad/common/embedded_files.cpp:474`
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:3048`
- Internal parser path:
  an unknown child after `name` inside an embedded file is tolerated by the schematic-level
  embedded-files wrapper and does not surface as a visible diagnostic
- Observation:
  `(embedded_files (file (name foo.txt) (bogus)))` still exports through `kicad-cli` with exit 0
  and no visible diagnostics, and `ki extract` matches that accepted form
- Severity: none
- Export behavior: succeeds

### `embedded_files_file_type_bogus_ok`

- Relevant KiCad sources:
  - `/Users/Daniel/Desktop/kicad/common/embedded_files.cpp:474`
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:3048`
- Internal parser path:
  an invalid `type` token inside an embedded file is tolerated by the schematic-level
  embedded-files wrapper and does not surface as a visible diagnostic
- Observation:
  `(embedded_files (file (name foo.txt) (type bogus)))` still exports through `kicad-cli` with
  exit 0 and no visible diagnostics, and `ki extract` matches that accepted form
- Severity: none
- Export behavior: succeeds

### `embedded_files_file_unknown_after_name`

- Relevant KiCad sources:
  - `/Users/Daniel/Desktop/kicad/common/embedded_files.cpp:482`
- Internal parser path:
  an embedded `file` entry tolerates some bare unknown children, but an unknown child with payload
  after `name` still fails the parse
- Observation:
  `(embedded_files (file (name foo.txt) (bogus 1)))` fails through `kicad-cli` and surfaces only
  the generic message `Failed to load schematic`
- Severity: error
- Export behavior: fails before export

### `embedded_files_file_unknown_before_name`

- Relevant KiCad sources:
  - `/Users/Daniel/Desktop/kicad/common/embedded_files.cpp:482`
- Internal parser path:
  an embedded `file` entry also fails when an unknown payload-bearing child appears before `name`
- Observation:
  `(embedded_files (file (bogus 1) (name foo.txt)))` fails through `kicad-cli` and surfaces only
  the generic message `Failed to load schematic`
- Severity: error
- Export behavior: fails before export

### `future_sch_bare`

- Relevant KiCad sources:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:3057`
- Internal parser path:
  `future_sch` is not a loadable top-level schematic object on the export path
- Observation:
  a schematic containing bare `(future_sch)` fails through `kicad-cli` and surfaces only the
  generic message `Failed to load schematic`
- Severity: error
- Export behavior: fails before export

### `future_sch_payload`

- Relevant KiCad sources:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:3057`
- Internal parser path:
  `future_sch` is not a loadable top-level schematic object on the export path, even with payload
- Observation:
  a schematic containing `(future_sch (bogus 1))` fails through `kicad-cli` and surfaces only the
  generic message `Failed to load schematic`
- Severity: error
- Export behavior: fails before export

### `private_sheet_name_ok`

- Relevant KiCad sources:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:2289`
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:2353`
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:3759`
- Internal parser path:
  `parseSchField()` accepts the `private` prefix before the legacy sheet mandatory-field alias
  `Sheet name`
- Observation:
  `property private "Sheet name" ...` on a schematic sheet exports through `kicad-cli` with exit 0
  and no visible diagnostics; `ki extract` now matches that accepted form
- Severity: none
- Export behavior: succeeds

### `private_intersheetrefs_ok`

- Relevant KiCad sources:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:2289`
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:2370`
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:4744`
- Internal parser path:
  `parseSchField()` accepts the `private` prefix before the legacy global-label alias
  `Intersheet References`
- Observation:
  `property private "Intersheet References" ...` on a global label exports through `kicad-cli`
  with exit 0 and no visible diagnostics, and `ki extract` matches that accepted form
- Severity: none
- Export behavior: succeeds

### `text_box_duplicate_end_ok`

- Relevant KiCad sources:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:4835`
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:4840`
- Internal parser path:
  legacy `text_box/start` and repeated `text_box/end` are accepted on the modern text-box loader
- Observation:
  a `text_box` with repeated legacy `end` children still exports through `kicad-cli` with exit 0
  and no visible diagnostics, and `ki extract` matches that accepted form
- Severity: none
- Export behavior: succeeds

### `text_box_background`

- Relevant KiCad sources:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:4866`
- Internal parser path:
  schematic `text_box` rejects the `background` child; only `fill` is valid on this path
- Observation:
  a `text_box` carrying `background` fails through `kicad-cli` and surfaces only the generic
  message `Failed to load schematic`
- Severity: error
- Export behavior: fails before export

### `text_box_length`

- Relevant KiCad sources:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:4866`
- Internal parser path:
  schematic `text_box` rejects directive-label-only `length`
- Observation:
  a `text_box` carrying `length` fails through `kicad-cli` and surfaces only the generic message
  `Failed to load schematic`
- Severity: error
- Export behavior: fails before export

### `text_box_fields_autoplaced`

- Relevant KiCad sources:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:4866`
- Internal parser path:
  schematic `text_box` rejects directive-label-only `fields_autoplaced`
- Observation:
  a `text_box` carrying `fields_autoplaced` fails through `kicad-cli` and surfaces only the
  generic message `Failed to load schematic`
- Severity: error
- Export behavior: fails before export

### `text_box_iref`

- Relevant KiCad sources:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:4866`
- Internal parser path:
  schematic `text_box` rejects label-only `iref`
- Observation:
  a `text_box` carrying `iref` fails through `kicad-cli` and surfaces only the generic message
  `Failed to load schematic`
- Severity: error
- Export behavior: fails before export

### `effects_unknown_child`

- Relevant KiCad sources:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:897`
- Internal parser path:
  `effects` rejects unknown child lists beyond `font`, `justify`, `hide`, and `href`
- Observation:
  a text `effects` block carrying `(bogus 1)` fails through `kicad-cli` and surfaces only the
  generic message `Failed to load schematic`
- Severity: error
- Export behavior: fails before export

### `fill_unknown_child`

- Relevant KiCad sources:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:759`
- Internal parser path:
  `fill` rejects unknown child lists beyond `type` and `color`
- Observation:
  a `fill` block carrying `(bogus 1)` fails through `kicad-cli` and surfaces only the generic
  message `Failed to load schematic`
- Severity: error
- Export behavior: fails before export

### `stroke_unknown_child`

- Relevant KiCad sources:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:737`
- Internal parser path:
  `stroke` rejects unknown child lists beyond `width`, `type`, and `color`
- Observation:
  a `stroke` block carrying `(bogus 1)` fails through `kicad-cli` and surfaces only the generic
  message `Failed to load schematic`
- Severity: error
- Export behavior: fails before export

### `fill_bare_atom`

- Relevant KiCad sources:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:718`
- Internal parser path:
  `fill` requires child lists and rejects a bare atom payload
- Observation:
  a `fill` block written as `(fill foo)` fails through `kicad-cli` and surfaces only the generic
  message `Failed to load schematic`
- Severity: error
- Export behavior: fails before export

### `fill_color_only`

- Relevant KiCad sources:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:706`
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:744`
- Internal parser path:
  `parseFill()` initializes defaults and accepts a `color` child without requiring an explicit
  `type`
- Observation:
  a sheet using `fill (color 0 0 0 0)` exports through `kicad-cli` with exit code `0` and no
  visible diagnostics
- Parity implication:
  `ki extract` should accept color-only `fill` blocks, not just `type`-bearing forms

### `stroke_bare_atom`

- Relevant KiCad sources:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:798`
- Internal parser path:
  `stroke` requires child lists and rejects a bare atom payload
- Observation:
  a `stroke` block written as `(stroke foo)` fails through `kicad-cli` and surfaces only the
  generic message `Failed to load schematic`
- Severity: error
- Export behavior: fails before export

### `stroke_color_only`

- Relevant KiCad source:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:696`
- Internal parser path:
  `parseStroke()` accepts sparse stroke blocks, including a lone `color` child
- Observation:
  a sheet using `stroke (color 0 0 0 0)` exports through `kicad-cli` with exit code `0` and no
  visible diagnostics
- Parity implication:
  `ki extract` should accept color-only `stroke` blocks

### `effects_font_bare_atom`

- Relevant KiCad sources:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:847`
- Internal parser path:
  `font` inside `effects` requires child lists and rejects a bare atom payload
- Observation:
  `(effects (font foo))` fails through `kicad-cli` and surfaces only the generic message
  `Failed to load schematic`
- Severity: error
- Export behavior: fails before export

### `effects_font_unknown_child`

- Relevant KiCad sources:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:847`
- Internal parser path:
  `font` inside `effects` rejects unknown child lists beyond the standard font fields
- Observation:
  `(effects (font (size 1.27 1.27) (bogus 1)))` fails through `kicad-cli` and surfaces only the
  generic message `Failed to load schematic`
- Severity: error
- Export behavior: fails before export

### `symbol_property_unknown_child`

- Relevant KiCad sources:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:1157`
- Internal parser path:
  symbol properties reject unknown child lists beyond the standard property payload fields
- Observation:
  a placed-symbol `property` carrying `(bogus 1)` fails through `kicad-cli` and surfaces only the
  generic message `Failed to load schematic`
- Severity: error
- Export behavior: fails before export

### `fill_color_bare_atom`

- Relevant KiCad sources:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:718`
- Internal parser path:
  `fill/color` requires numeric atom children and rejects a bare symbol payload
- Observation:
  `(fill (color foo))` fails through `kicad-cli` and surfaces only the generic message
  `Failed to load schematic`
- Severity: error
- Export behavior: fails before export

### `stroke_color_bare_atom`

- Relevant KiCad sources:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:798`
- Internal parser path:
  `stroke/color` requires numeric atom children and rejects a bare symbol payload
- Observation:
  `(stroke (color foo))` fails through `kicad-cli` and surfaces only the generic message
  `Failed to load schematic`
- Severity: error
- Export behavior: fails before export

### `property_bare_atom_child`

- Relevant KiCad sources:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:1157`
- Internal parser path:
  a property rejects a trailing bare atom child after its standard field payload
- Observation:
  a property written with trailing `foo` fails through `kicad-cli` and surfaces only the generic
  message `Failed to load schematic`
- Severity: error
- Export behavior: fails before export

### `text_box_margins_bare_atom`

- Relevant KiCad sources:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:4824`
- Internal parser path:
  `text_box/margins` requires numeric payload and rejects a bare symbol atom
- Observation:
  `(text_box ... (margins foo) ...)` fails through `kicad-cli` and surfaces only the generic
  message `Failed to load schematic`
- Severity: error
- Export behavior: fails before export

### `table_border_bare_atom`

- Relevant KiCad sources:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:5019`
- Internal parser path:
  `table/border` requires nested config children and rejects a bare atom payload
- Observation:
  `(table ... (border foo) ...)` fails through `kicad-cli` and surfaces only the generic message
  `Failed to load schematic`
- Severity: error
- Export behavior: fails before export

### `table_separators_bare_atom`

- Relevant KiCad sources:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:5052`
- Internal parser path:
  `table/separators` requires nested config children and rejects a bare atom payload
- Observation:
  `(table ... (separators foo) ...)` fails through `kicad-cli` and surfaces only the generic
  message `Failed to load schematic`
- Severity: error
- Export behavior: fails before export

### `table_cells_bare_atom`

- Relevant KiCad sources:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:4986`
- Internal parser path:
  `table/cells` requires `table_cell` list entries and rejects a bare atom payload
- Observation:
  `(table ... (cells foo) ...)` fails through `kicad-cli` and surfaces only the generic message
  `Failed to load schematic`
- Severity: error
- Export behavior: fails before export

### `table_row_heights_bare_atom`

- Relevant KiCad sources:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:5066`
- Internal parser path:
  `table/row_heights` requires numeric entries and rejects a bare symbol atom
- Observation:
  `(table ... (row_heights foo) ...)` fails through `kicad-cli` and surfaces only the generic
  message `Failed to load schematic`
- Severity: error
- Export behavior: fails before export

### `table_col_widths_bare_atom`

- Relevant KiCad sources:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:5066`
- Internal parser path:
  `table/col_widths` requires numeric entries and rejects a bare symbol atom
- Observation:
  `(table ... (col_widths foo) ...)` fails through `kicad-cli` and surfaces only the generic
  message `Failed to load schematic`
- Severity: error
- Export behavior: fails before export

### `table_columns_bare_atom`

- Relevant KiCad sources:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:5066`
- Internal parser path:
  `table/columns` requires a numeric value and rejects a bare symbol atom
- Observation:
  `(table ... (columns foo) ...)` fails through `kicad-cli` and surfaces only the generic message
  `Failed to load schematic`
- Severity: error
- Export behavior: fails before export

### `table_rows_bare_atom`

- Relevant KiCad sources:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:5066`
- Internal parser path:
  `table/rows` requires a numeric value and rejects a bare symbol atom
- Observation:
  `(table ... (rows foo) ...)` fails through `kicad-cli` and surfaces only the generic message
  `Failed to load schematic`
- Severity: error
- Export behavior: fails before export

### `table_header_bare_atom`

- Relevant KiCad sources:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:5066`
- Internal parser path:
  `table/header` requires a boolean payload and rejects a bare symbol atom
- Observation:
  `(table ... (header foo) ...)` fails through `kicad-cli` and surfaces only the generic message
  `Failed to load schematic`
- Severity: error
- Export behavior: fails before export

### `table_border_external_bare_atom`

- Relevant KiCad sources:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:5019`
- Internal parser path:
  `table/border/external` requires a boolean payload and rejects a bare symbol atom
- Observation:
  `(table ... (border (external foo)) ...)` fails through `kicad-cli` and surfaces only the
  generic message `Failed to load schematic`
- Severity: error
- Export behavior: fails before export

### `table_separators_rows_bare_atom`

- Relevant KiCad sources:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:5052`
- Internal parser path:
  `table/separators/rows` requires a boolean payload and rejects a bare symbol atom
- Observation:
  `(table ... (separators (rows foo)) ...)` fails through `kicad-cli` and surfaces only the
  generic message `Failed to load schematic`
- Severity: error
- Export behavior: fails before export

### `table_separators_cols_bare_atom`

- Relevant KiCad sources:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:5052`
- Internal parser path:
  `table/separators/cols` requires a boolean payload and rejects a bare symbol atom
- Observation:
  `(table ... (separators (cols foo)) ...)` fails through `kicad-cli` and surfaces only the
  generic message `Failed to load schematic`
- Severity: error
- Export behavior: fails before export

### `table_border_header_bare_atom`

- Relevant KiCad sources:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:5019`
- Internal parser path:
  `table/border/header` requires a boolean payload and rejects a bare symbol atom
- Observation:
  `(table ... (border (header foo)) ...)` fails through `kicad-cli` and surfaces only the generic
  message `Failed to load schematic`
- Severity: error
- Export behavior: fails before export

### `table_border_stroke_bare_atom`

- Relevant KiCad sources:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:5019`
- Internal parser path:
  `table/border/stroke` requires a stroke config list and rejects a bare symbol atom
- Observation:
  `(table ... (border (stroke foo)) ...)` fails through `kicad-cli` and surfaces only the generic
  message `Failed to load schematic`
- Severity: error
- Export behavior: fails before export

### `table_separators_stroke_bare_atom`

- Relevant KiCad sources:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:5052`
- Internal parser path:
  `table/separators/stroke` requires a stroke config list and rejects a bare symbol atom
- Observation:
  `(table ... (separators (stroke foo)) ...)` fails through `kicad-cli` and surfaces only the
  generic message `Failed to load schematic`
- Severity: error
- Export behavior: fails before export

### `bus_alias_members_list_child`

- Relevant KiCad sources:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:5104`
- Internal parser path:
  `bus_alias/members` rejects nested list children in place of member atoms
- Observation:
  `(bus_alias A (members (foo)))` fails through `kicad-cli` and surfaces only the generic message
  `Failed to load schematic`
- Severity: error
- Export behavior: fails before export

### `sheet_instances_extra_bare_atom`

- Relevant KiCad sources:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:2589`
- Internal parser path:
  `sheet_instances/path` rejects trailing bare atoms after known children
- Observation:
  `(sheet_instances (path "/" (page "1") foo))` fails through `kicad-cli` and surfaces only the
  generic message `Failed to load schematic`
- Severity: error
- Export behavior: fails before export

### `symbol_instances_extra_bare_atom`

- Relevant KiCad sources:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:2688`
- Internal parser path:
  `symbol_instances/path` rejects trailing bare atoms after known children
- Observation:
  `(symbol_instances (path "/abc" (reference "R1") foo))` fails through `kicad-cli` and surfaces
  only the generic message `Failed to load schematic`
- Severity: error
- Export behavior: fails before export

### `default_instance_extra_bare_atom`

- Relevant KiCad sources:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:3480`
- Internal parser path:
  `default_instance` rejects trailing bare atoms after its known fields
- Observation:
  `(default_instance (reference "R1") foo)` fails through `kicad-cli` and surfaces only the
  generic message `Failed to load schematic`
- Severity: error
- Export behavior: fails before export

### `variant_extra_bare_atom`

- Relevant KiCad sources:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:3470`
- Internal parser path:
  `variant` rejects trailing bare atoms after its known fields
- Observation:
  `(variant (name "v") foo)` fails through `kicad-cli` and surfaces only the generic message
  `Failed to load schematic`
- Severity: error
- Export behavior: fails before export

### `nested_instance_project_extra_bare_atom`

- Relevant KiCad sources:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:3330`
- Internal parser path:
  `instances/project` rejects trailing bare atoms after its known children
- Observation:
  `(instances (project "proj" foo))` fails through `kicad-cli` and surfaces only the generic
  message `Failed to load schematic`
- Severity: error
- Export behavior: fails before export

### `group_extra_bare_atom`

- Relevant KiCad sources:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:5159`
- Internal parser path:
  `group` rejects a trailing bare atom after its optional quoted name
- Observation:
  `(group "g" foo)` fails through `kicad-cli` and surfaces only the generic message
  `Failed to load schematic`
- Severity: error
- Export behavior: fails before export

### `sheet_property_bare_atom_child`

- Relevant KiCad sources:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:1157`
- Internal parser path:
  a sheet property rejects a trailing bare atom child after its standard payload
- Observation:
  a sheet `property` written with trailing `foo` fails through `kicad-cli` and surfaces only the
  generic message `Failed to load schematic`
- Severity: error
- Export behavior: fails before export

### `image_data_list_child`

- Relevant KiCad sources:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:3643`
- Internal parser path:
  `image/data` rejects nested list children in place of base64 atom fragments
- Observation:
  `(image ... (data (foo)))` fails through `kicad-cli` and surfaces only the generic message
  `Failed to load schematic`
- Severity: error
- Export behavior: fails before export

### `group_locked_bare_atom`

- Relevant KiCad sources:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:5159`
- Internal parser path:
  `group/locked` requires a boolean payload and rejects a bare symbol atom
- Observation:
  `(group "g" (locked foo))` fails through `kicad-cli` and surfaces only the generic message
  `Failed to load schematic`
- Severity: error
- Export behavior: fails before export

### `group_name_list_child`

- Relevant KiCad sources:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:5111`
- Internal parser path:
  `group` rejects a nested list in place of its optional quoted name
- Observation:
  `(group (foo))` fails through `kicad-cli` and surfaces only the generic message
  `Failed to load schematic`
- Severity: error
- Export behavior: fails before export

### `bus_alias_name_list_child`

- Relevant KiCad sources:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:5111`
- Internal parser path:
  `bus_alias` rejects a nested list in place of its quoted-string name
- Observation:
  `(bus_alias (foo) (members A))` fails through `kicad-cli` and surfaces only the generic message
  `Failed to load schematic`
- Severity: error
- Export behavior: fails before export

### `embedded_files_file_name_list_child`

- Relevant KiCad sources:
  - `/Users/Daniel/Desktop/kicad/common/embedded_files.cpp:452`
- Internal parser path:
  an embedded file `name` requires an atom payload and rejects a nested list child
- Observation:
  `(embedded_files (file (name (foo))))` fails through `kicad-cli` and surfaces only the generic
  message `Failed to load schematic`
- Severity: error
- Export behavior: fails before export

### `class_label_bare`

- Relevant KiCad sources:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:3057`
- Internal parser path:
  `class_label` is listed in the top-level token set but still fails on the export/load path
- Observation:
  a schematic containing bare `(class_label)` fails through `kicad-cli` and surfaces only the
  generic message `Failed to load schematic`
- Severity: error
- Export behavior: fails before export

### `bitmap_data_list_child`

- Relevant KiCad sources:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:3057`
- Internal parser path:
  the legacy top-level `bitmap` token also fails when carrying nested list image data
- Observation:
  `(bitmap (at 10 10) (data (foo)))` fails through `kicad-cli` and surfaces only the generic
  message `Failed to load schematic`
- Severity: error
- Export behavior: fails before export

### `symbol_pin_alternate_bare_atom`

- Relevant KiCad sources:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:1800`
- Internal parser path:
  a library-symbol pin `alternate` entry rejects a bare atom payload in place of the expected
  name/type/shape tuple
- Observation:
  `(alternate foo)` on a library-symbol pin fails through `kicad-cli` and surfaces only the
  generic message `Failed to load schematic`
- Severity: error
- Export behavior: fails before export

### `symbol_pin_name_extra_bare_atom`

- Relevant KiCad sources:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:1727`
- Internal parser path:
  a library-symbol pin `name` rejects trailing bare atoms after its `effects`
- Observation:
  a pin `name` written with trailing `foo` fails through `kicad-cli` and surfaces only the generic
  message `Failed to load schematic`
- Severity: error
- Export behavior: fails before export

### `symbol_pin_number_extra_bare_atom`

- Relevant KiCad sources:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:1767`
- Internal parser path:
  a library-symbol pin `number` rejects trailing bare atoms after its `effects`
- Observation:
  a pin `number` written with trailing `foo` fails through `kicad-cli` and surfaces only the
  generic message `Failed to load schematic`
- Severity: error
- Export behavior: fails before export

### `schematic_symbol_pin_extra_bare_atom`

- Relevant KiCad sources:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:3568`
- Internal parser path:
  a placed-symbol `pin` entry rejects trailing bare atoms after its pin number
- Observation:
  `(pin "1" foo)` on a placed symbol fails through `kicad-cli` and surfaces only the generic
  message `Failed to load schematic`
- Severity: error
- Export behavior: fails before export

### `effects_justify_list_child`

- Relevant KiCad sources:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:864`
- Internal parser path:
  `effects/justify` rejects nested list children in place of alignment atoms
- Observation:
  `(effects (justify (foo)) ...)` fails through `kicad-cli` and surfaces only the generic message
  `Failed to load schematic`
- Severity: error
- Export behavior: fails before export

### `pin_angles_bare_atom`

- Relevant KiCad sources:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:1333`
- Internal parser path:
  a library-symbol pin `angles` entry rejects a bare symbol atom
- Observation:
  `(angles foo)` on a library-symbol pin fails through `kicad-cli` and surfaces only the generic
  message `Failed to load schematic`
- Severity: error
- Export behavior: fails before export

### `variant_field_name_list_child`

- Relevant KiCad sources:
  - `/Users/Daniel/Desktop/kicad/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr_parser.cpp:3461`
- Internal parser path:
  a `variant/field` rejects a nested list in place of the field name atom
- Observation:
  `(field (foo) "bar")` inside a variant fails through `kicad-cli` and surfaces only the generic
  message `Failed to load schematic`
- Severity: error
- Export behavior: fails before export

Observed accepted forms on this branch:
- `property private "Sheet name" ...` still exports
- `property private "Intersheet References" ...` still exports
- `embedded_files (bogus)` still exports
- `(embedded_files (file))` still exports
- `(embedded_files (file (name foo.txt) (bogus)))` still exports
- `(embedded_files (file (name foo.txt) (type bogus)))` still exports
- `(embedded_files (file (name foo.txt) (data abc)))` still exports
- `(future_sch (bogus 1))` also collapses to the same generic failed-load result
- `table_cell` also rejects `background`, and `ki` already matches that path
- `text_box` duplicate legacy `end` still exports
- plain `fill (type none)` and standard text `effects` still export normally
- `effects foo` already fails in both KiCad and `ki`
- `effects (font (size 1.27 1.27))` still exports normally
- standard symbol properties with only `at` and `effects` still export normally
- `group (uuid foo)` still exports in both KiCad and `ki`
- valid placed-symbol `default_instance` fallback now exports through `ki extract`, including the
  `default_instance_valid_only` parity case
- valid plain schematic `text` now exports through `ki extract`, including repeated `effects`
- valid plain schematic `text_box` now exports through `ki extract`, including valid
  `(margins ...)` and repeated `effects`
- `netclass_flag` accepts generic `property` names like `"P"` as well as `"Netclass"` and
  `"Component Class"`
- top-level `bezier (pts)` still exports
