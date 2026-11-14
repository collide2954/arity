<script lang="ts">
  import { link } from "../lib/route";
</script>

<h1>About arity</h1>

<p class="lede">
  arity is a database of the values and effects that open-access research articles report. It keeps
  each value with its full provenance, so that each person can see where a value comes from and
  compare it with the source.
</p>

<h2>Principles</h2>
<ul>
  <li>
    <strong>Reports only.</strong> Each record is a report of what one source states. arity does not rate,
    rank, combine, or correct reports, and it does not calculate consensus values.
  </li>
  <li>
    <strong>Provenance for each value.</strong> Each report keeps the source, the place in the source,
    the exact quote, the reader, the responsible person, and the software versions.
  </li>
  <li>
    <strong>Checks before trust.</strong> Software checks each submission against the source text. Then
    a person compares each report with its highlighted source and confirms or rejects it.
  </li>
  <li>
    <strong>Notices stay visible.</strong> arity records retractions, corrections, and expressions of
    concern from each source that reports them, and shows them with the data. It does not remove data.
  </li>
  <li>
    <strong>Open data.</strong> You can download the full database as one SQLite file.
  </li>
</ul>

<h2>How a value comes into arity</h2>
<ol>
  <li>arity finds open-access articles in Europe PMC and adds them to a queue.</li>
  <li>
    arity downloads the full text, the PDF, and the supplementary files from the PMC open-access
    store of NCBI, and records the address, the date, and the SHA-256 digest of each file.
  </li>
  <li>
    A reader reads the canonical text and writes a submission. The reader can be a person or a
    software agent. Each submission names the reader and the person who is responsible for it.
  </li>
  <li>
    arity checks the submission. Each quote must occur in its block of the text. Each number must
    occur in a quote. Each unit must agree with its property and with the unit text in the source.
    Each identifier must be valid for the kind of entity. A submission with an error does not go
    into the database.
  </li>
  <li>A person compares each report with its source and records the result.</li>
</ol>

<h2>Values that arity calculates</h2>
<p>
  arity shows each value as the source reports it. To compare values, arity also converts each value
  to the unit of its property. The website marks these values as calculated. The conversion uses the
  UCUM unit definitions.
</p>

<h2>Licenses</h2>
<p>
  The code of arity is under the Apache License 2.0. The data that arity publishes is under CC0 1.0.
  Each source keeps its own license; arity shows the text and the PDF of a source only when the
  license of the source lets others share them.
</p>

<h2>Source code</h2>
<p>
  <a href="https://github.com/collide2954/arity">github.com/collide2954/arity</a>. To report an
  error in the data, open an issue with the number of the report and the text of the source.
</p>

<p><a href={link({ page: "home" })}>Go to the start page</a></p>
