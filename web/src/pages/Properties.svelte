<script lang="ts">
  import { loadIndex } from "../lib/data";
  import { unitLabel } from "../lib/format";
  import { link } from "../lib/route";
</script>

<h1>Properties</h1>
<p class="lede">
  Each property is a quantity or a quality that reports give values for. Select a property to
  compare its values across works, experiments, and conditions.
</p>

{#await loadIndex()}
  <p class="muted">arity is loading the properties.</p>
{:then index}
  <div class="table-wrap card">
    <table>
      <thead>
        <tr
          ><th>Property</th><th>Key</th><th>Type</th><th>Display unit</th><th>Dimension</th><th
            >Reports</th
          ></tr
        >
      </thead>
      <tbody>
        {#each index.properties as property (property.key)}
          <tr>
            <td><a href={link({ page: "property", key: property.key })}>{property.label}</a></td>
            <td class="mono">{property.key}</td>
            <td>{property.value_type}</td>
            <td>{property.unit ? unitLabel(property.unit) || "1" : ""}</td>
            <td class="mono">{property.dimension ?? ""}</td>
            <td>{property.reports}</td>
          </tr>
        {/each}
      </tbody>
    </table>
  </div>
{/await}
