<script lang="ts">
  import { loadIndex } from "../lib/data";
  import { link } from "../lib/route";
</script>

<h1>Entities</h1>
<p class="lede">
  The things that reports are about. arity links an entity across works through its identifiers in
  public registries.
</p>

{#await loadIndex()}
  <p class="muted">arity is loading the entities.</p>
{:then index}
  <div class="table-wrap card">
    <table>
      <thead>
        <tr
          ><th>Entity</th><th>Kind</th><th>Identifiers</th><th>Names in sources</th><th>Reports</th
          ></tr
        >
      </thead>
      <tbody>
        {#each index.entities as entity (entity.id)}
          <tr>
            <td><a href={link({ page: "entity", id: entity.id })}>{entity.label}</a></td>
            <td>{entity.kind.replaceAll("_", " ")}</td>
            <td>
              <div class="chips">
                {#each entity.xrefs as xref (xref)}
                  <a class="chip mono" href="https://bioregistry.io/{xref}">{xref}</a>
                {/each}
              </div>
            </td>
            <td>{entity.names.join(", ")}</td>
            <td>{entity.reports}</td>
          </tr>
        {/each}
      </tbody>
    </table>
  </div>
{/await}
