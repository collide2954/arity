# Issue codes

<!-- The tests of arity-protocol write this file. After a change, run them with ARITY_UPDATE_DOCS=1. -->

The checks give each problem one of these codes. An error stops the submission. A warning lets the submission go into the database, and a person must look at the part that the warning names. `arity vocab issues` prints this list.

| Code | Severity | Meaning |
| --- | --- | --- |
| `ambiguous_quote` | warning | The quote occurs more than one time in the block, and the submission does not give the occurrence. arity uses the first. |
| `bound_cue_missing` | warning | The value has a bound, but no quote has a sign or a word for that bound. |
| `category_not_in_evidence` | warning | No quote of the report contains the category of the value. |
| `condition_shape` | error | A condition does not have a role with an entity or a text, or a property with a value. |
| `dimension_mismatch` | error | The unit does not measure the dimension of the property. |
| `direction_cue_missing` | warning | No quote has a word that states the direction of the effect. |
| `duplicate_id` | error | Two entities, or two reports, have the same identifier. |
| `empty_label` | error | The label of an entity is empty. |
| `empty_method` | error | A value from a figure does not tell how the reader read it. |
| `empty_name` | error | A name that the submission must give is empty, for example the name of the reader. |
| `empty_quote` | error | A quote is empty. |
| `figure_not_found` | error | The text has no figure caption with the key that the value names. |
| `figure_not_in_evidence` | error | A value from a figure does not have the caption of the figure in its evidence. |
| `invalid_id` | error | An identifier is empty or has a space. |
| `invalid_interval` | error | The low end of an interval is greater than the high end. |
| `invalid_json` | error | The file is not JSON, or it does not have the fields of a submission. |
| `invalid_key` | error | A property key does not start with a letter, or has characters other than lower-case letters, digits, and underscores. |
| `invalid_level` | error | The level of an interval is not a percent from 0 to 100. |
| `invalid_p` | error | A p-value is not from 0 to 1. |
| `invalid_resolution` | error | The resolution of a reading from a figure is not more than zero. |
| `invalid_sample_size` | error | The number of samples is less than 1. |
| `invalid_unit` | error | The unit is not a valid UCUM unit. |
| `label_differs` | warning | The label of a property is not the stored label. arity keeps the stored label. |
| `long_quote` | warning | A quote is long. A short quote is easier to check. |
| `method_not_in_evidence` | warning | No quote names the method of the report. |
| `missing_definition` | error | The property is new, and the submission does not define it fully. |
| `negative_uncertainty` | error | An uncertainty is less than zero. |
| `no_agents` | error | An effect has no agent. |
| `no_evidence` | error | A report has no quote from the source. |
| `no_mention` | error | An entity has no quote of a place where the source names it. |
| `no_reports` | error | The submission has no reports and no skipped quotes. |
| `no_subject` | error | A value has no subject. |
| `no_version` | warning | The software that read the source has no version. |
| `no_xref` | warning | The entity has no registry identifier, so arity cannot link it to other works. |
| `number_not_in_evidence` | error | No quote of the report contains the number. |
| `occurrence_out_of_range` | error | The quote does not occur as many times in the block as the occurrence that the submission gives. |
| `property_conflict` | error | The definition of a property does not agree with the stored definition. |
| `quote_not_found` | error | The quote does not occur in the block. |
| `quote_too_long` | error | A quote is longer than the maximum length of a quote. |
| `skip_note_required` | error | A quote that the reader skips for the reason `other` has no note that tells the reason. |
| `skip_without_number` | error | A quote that the reader skips contains no number. |
| `significance_shape` | error | A significance does not have a statement, a p-value, or both. |
| `text_mismatch` | error | The submission names a text that is not the current text of the work. |
| `text_not_in_evidence` | error | No quote contains a text that the report gives, for example the comparator. |
| `text_unreadable` | error | arity cannot make the digest of the text. |
| `uncertainty_shape` | error | An uncertainty does not have a number, or a low and a high end. |
| `unexpected_field` | error | The submission gives a field that does not apply, for example a unit for a property that is not a quantity. |
| `unit_required` | error | A quantity has no UCUM unit. |
| `unit_text_mismatch` | error | The unit that the source writes is not the same unit as the UCUM unit. |
| `unit_text_not_in_evidence` | error | No quote contains the unit as the source writes it. |
| `unit_text_required` | error | A quantity does not give the unit as the source writes it. |
| `unit_text_unreadable` | warning | arity cannot read the unit that the source writes, so a person must check the unit. |
| `unknown_block` | error | The text has no block with this key. |
| `unknown_category` | error | The category is not one of the categories of the property. |
| `unknown_entity` | error | No entity of the submission has this identifier. |
| `unknown_format` | error | The format of the submission is not arity-submission/1. |
| `unknown_property` | error | The property is not in the database, and the submission does not define it. |
| `value_shape` | error | A value does not have exactly one of: a number, a low and a high end, a boolean, or a category. |
| `value_type_mismatch` | error | The value does not have the value type of the property. |
| `xref_kind_mismatch` | error | The registry of the identifier does not identify entities of this kind. |
