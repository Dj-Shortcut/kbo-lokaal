const { invoke } = window.__TAURI__.core;
const { open } = window.__TAURI__.dialog;
const $ = (id) => document.getElementById(id);
const esc = (value) => String(value ?? '').replace(/[&<>"']/g, (c) => ({'&':'&amp;','<':'&lt;','>':'&gt;','"':'&quot;',"'":'&#39;'}[c]));

function render(result) {
  $('message').textContent = `${result.totaal.toLocaleString('nl-BE')} resultaten gevonden. Eerste ${result.rijen.length} getoond.`;
  if (!result.rijen.length) { $('results').innerHTML = '<div class="empty">Geen ondernemingen gevonden.</div>'; return; }
  const rows = result.rijen.map((r) => `<tr><td>${esc(r.kbo_nummer)}</td><td>${esc(r.naam)}</td><td>${esc(r.adres)}</td><td>${esc(r.contacten)}</td><td>${esc(r.activiteiten)}</td><td>${esc(r.startdatum)}</td></tr>`).join('');
  $('results').innerHTML = `<div class="table-wrap"><table><thead><tr><th>KBO-nummer</th><th>Naam</th><th>Adres</th><th>Contacten</th><th>Activiteiten</th><th>Startdatum</th></tr></thead><tbody>${rows}</tbody></table></div>`;
}

async function search() {
  $('message').textContent = 'Zoeken…';
  try {
    const result = await invoke('search_companies', { request: request() });
    render(result);
  } catch (error) { $('message').textContent = `Fout: ${error}`; $('results').innerHTML = ''; }
}

function request() { return Object.fromEntries(['db_path','naam','gemeente','postcode','straat','email','contact','activiteit','kbo','juridisch','start_van','start_tot','status'].map((id) => [id, $(id).value.trim()])); }

$('import-zip').addEventListener('click', async () => {
  const selected = await open({ multiple: false, directory: false, filters: [{ name: 'KBO zipbestand', extensions: ['zip'] }] });
  if (!selected) return; $('message').textContent = 'KBO-zip wordt lokaal geïmporteerd; dit kan lang duren…';
  try { $('db-path').value = await invoke('import_kbo_zip', { zipPath: selected }); localStorage.setItem('kbo-db-path', $('db-path').value); $('message').textContent = 'Import klaar. Kies je filters en zoek.'; }
  catch (error) { $('message').textContent = `Import mislukt: ${error}`; }
});

$('export').addEventListener('click', async () => {
  $('message').textContent = 'CSV wordt opgebouwd…';
  try { const csv = await invoke('export_csv', { request: request() }); const blob = new Blob([csv], {type:'text/csv;charset=utf-8'}); const a=document.createElement('a'); a.href=URL.createObjectURL(blob); a.download='kbo_prospectlijst.csv'; a.click(); URL.revokeObjectURL(a.href); $('message').textContent = 'CSV-export klaar.'; }
  catch (error) { $('message').textContent = `Export mislukt: ${error}`; }
});

$('search').addEventListener('click', search);
$('choose-db').addEventListener('click', async () => {
  const selected = await open({ multiple: false, directory: false, filters: [{ name: 'SQLite database', extensions: ['sqlite', 'db'] }] });
  if (selected) { $('db-path').value = selected; localStorage.setItem('kbo-db-path', selected); }
});
$('db-path').value = localStorage.getItem('kbo-db-path') || '';
$('db-path').addEventListener('change', () => localStorage.setItem('kbo-db-path', $('db-path').value));
