use csv::ReaderBuilder;
use rusqlite::{params_from_iter, types::Value, Connection};
use serde::{Deserialize, Serialize};
use std::{fs::File, path::{Path, PathBuf}};
use zip::ZipArchive;

#[derive(Debug, Deserialize, Clone)]
struct SearchRequest {
    db_path: String, naam: String, gemeente: String, postcode: String, straat: String,
    email: String, contact: String, activiteit: String, kbo: String, juridisch: String,
    start_van: String, start_tot: String, status: String,
}

#[derive(Debug, Serialize)]
struct Company { kbo_nummer: String, startdatum: String, naam: String, adres: String, contacten: String, activiteiten: String, ondernemings_type: String, rechtsvorm: String, status: String }
#[derive(Debug, Serialize)]
struct SearchResult { totaal: i64, rijen: Vec<Company> }

fn query_parts(request: &SearchRequest, limit: Option<usize>) -> (String, Vec<Value>) {
    let mut where_parts = vec!["1=1".to_string()]; let mut values = Vec::new();
    if request.status != "ALLE" { where_parts.push("e.Status = ?".into()); values.push(Value::Text(if request.status.is_empty() { "AC".into() } else { request.status.clone() })); }
    let add_like = |parts: &mut Vec<String>, vals: &mut Vec<Value>, condition: &str, value: &str| { if !value.trim().is_empty() { parts.push(condition.into()); vals.push(Value::Text(format!("%{}%", value.trim()))); } };
    if !request.naam.trim().is_empty() { where_parts.push("EXISTS (SELECT 1 FROM denomination d WHERE d.EntityNumber=e.EnterpriseNumber AND d.Denomination LIKE ? COLLATE NOCASE)".into()); values.push(Value::Text(format!("%{}%", request.naam.trim()))); }
    if !request.gemeente.trim().is_empty() { where_parts.push("EXISTS (SELECT 1 FROM address a WHERE a.EntityNumber=e.EnterpriseNumber AND (a.MunicipalityNL LIKE ? COLLATE NOCASE OR a.MunicipalityFR LIKE ? COLLATE NOCASE))".into()); let v=format!("%{}%",request.gemeente.trim()); values.push(Value::Text(v.clone())); values.push(Value::Text(v)); }
    add_like(&mut where_parts,&mut values,"EXISTS (SELECT 1 FROM address a WHERE a.EntityNumber=e.EnterpriseNumber AND a.Zipcode LIKE ?)",&request.postcode);
    if !request.straat.trim().is_empty() { where_parts.push("EXISTS (SELECT 1 FROM address a WHERE a.EntityNumber=e.EnterpriseNumber AND (a.StreetNL LIKE ? COLLATE NOCASE OR a.StreetFR LIKE ? COLLATE NOCASE))".into()); let v=format!("%{}%",request.straat.trim()); values.push(Value::Text(v.clone())); values.push(Value::Text(v)); }
    add_like(&mut where_parts,&mut values,"EXISTS (SELECT 1 FROM contact c WHERE c.EntityNumber=e.EnterpriseNumber AND lower(c.ContactType)='email' AND c.Value LIKE ? COLLATE NOCASE)",&request.email);
    add_like(&mut where_parts,&mut values,"EXISTS (SELECT 1 FROM contact c WHERE c.EntityNumber=e.EnterpriseNumber AND c.Value LIKE ? COLLATE NOCASE)",&request.contact);
    if !request.activiteit.trim().is_empty() { where_parts.push("EXISTS (SELECT 1 FROM activity x WHERE x.EntityNumber=e.EnterpriseNumber AND (x.NaceCode LIKE ? OR x.Classification LIKE ? COLLATE NOCASE OR EXISTS (SELECT 1 FROM code c WHERE c.Category='Nace' || x.NaceVersion AND c.Code=x.NaceCode AND c.Language='NL' AND c.Description LIKE ? COLLATE NOCASE)))".into()); let v=format!("%{}%",request.activiteit.trim()); values.push(Value::Text(v.clone())); values.push(Value::Text(v.clone())); values.push(Value::Text(v)); }
    if !request.kbo.trim().is_empty() { where_parts.push("e.EnterpriseNumber LIKE ?".into()); values.push(Value::Text(format!("%{}%",request.kbo.trim()))); }
    if !request.juridisch.trim().is_empty() { where_parts.push("(e.JuridicalForm LIKE ? OR e.TypeOfEnterprise LIKE ?)".into()); let v=format!("%{}%",request.juridisch.trim()); values.push(Value::Text(v.clone())); values.push(Value::Text(v)); }
    if !request.start_van.trim().is_empty() { where_parts.push("e.StartDate >= ?".into()); values.push(Value::Text(request.start_van.clone())); }
    if !request.start_tot.trim().is_empty() { where_parts.push("e.StartDate <= ?".into()); values.push(Value::Text(request.start_tot.clone())); }
    let limit_sql = limit.map(|n| format!(" LIMIT {n}")).unwrap_or_default();
    let base = format!("FROM enterprise e WHERE {}", where_parts.join(" AND "));
    let sql = format!("SELECT e.EnterpriseNumber,e.StartDate,(SELECT group_concat(DISTINCT d.Denomination) FROM denomination d WHERE d.EntityNumber=e.EnterpriseNumber),(SELECT group_concat(DISTINCT COALESCE(a.StreetNL,a.StreetFR)||' '||a.HouseNumber||', '||a.Zipcode||' '||COALESCE(a.MunicipalityNL,a.MunicipalityFR)) FROM address a WHERE a.EntityNumber=e.EnterpriseNumber),(SELECT group_concat(DISTINCT c.ContactType||': '||c.Value) FROM contact c WHERE c.EntityNumber=e.EnterpriseNumber),(SELECT group_concat(DISTINCT x.NaceCode||' - '||COALESCE((SELECT c.Description FROM code c WHERE c.Category='Nace'||x.NaceVersion AND c.Code=x.NaceCode AND c.Language='NL' LIMIT 1),'omschrijving onbekend')||' ('||x.Classification||')') FROM activity x WHERE x.EntityNumber=e.EnterpriseNumber),e.TypeOfEnterprise,e.JuridicalForm,e.Status {base} ORDER BY e.StartDate DESC{limit_sql}");
    (sql, values)
}

fn read_companies(connection: &Connection, sql: &str, values: &[Value]) -> Result<Vec<Company>, String> {
    let mut statement=connection.prepare(sql).map_err(|e|e.to_string())?;
    let rows=statement.query_map(params_from_iter(values.iter()),|row|Ok(Company{kbo_nummer:row.get(0)?,startdatum:row.get(1)?,naam:row.get::<_,Option<String>>(2)?.unwrap_or_default(),adres:row.get::<_,Option<String>>(3)?.unwrap_or_default(),contacten:row.get::<_,Option<String>>(4)?.unwrap_or_default(),activiteiten:row.get::<_,Option<String>>(5)?.unwrap_or_default(),ondernemings_type:row.get(6)?,rechtsvorm:row.get(7)?,status:row.get(8)?})).map_err(|e|e.to_string())?;
    rows.map(|r|r.map_err(|e|e.to_string())).collect()
}

fn resolved_db_path(path: &str) -> String {
    if !path.trim().is_empty() { return path.to_string(); }
    if let Ok(exe) = std::env::current_exe() {
        // Packaged layout: KBO Lokaal.app/Contents/MacOS/app, with the DB beside the .app.
        if let Some(folder) = exe.ancestors().nth(3) {
            let candidate = folder.join("kbo_open_data.sqlite");
            if candidate.exists() { return candidate.to_string_lossy().to_string(); }
        }
        if let Some(folder) = exe.parent() {
            let candidate = folder.join("kbo_open_data.sqlite");
            if candidate.exists() { return candidate.to_string_lossy().to_string(); }
        }
    }
    path.to_string()
}

#[tauri::command]
fn search_companies(request: SearchRequest) -> Result<SearchResult, String> {
    let connection=Connection::open(resolved_db_path(&request.db_path)).map_err(|e|format!("Database openen mislukt: {e}"))?;
    let (sql,values)=query_parts(&request,Some(100)); let count_sql=sql.replacen("SELECT e.EnterpriseNumber,e.StartDate,(SELECT group_concat(DISTINCT d.Denomination) FROM denomination d WHERE d.EntityNumber=e.EnterpriseNumber),(SELECT group_concat(DISTINCT COALESCE(a.StreetNL,a.StreetFR)||' '||a.HouseNumber||', '||a.Zipcode||' '||COALESCE(a.MunicipalityNL,a.MunicipalityFR)) FROM address a WHERE a.EntityNumber=e.EnterpriseNumber),(SELECT group_concat(DISTINCT c.ContactType||': '||c.Value) FROM contact c WHERE c.EntityNumber=e.EnterpriseNumber),(SELECT group_concat(DISTINCT x.NaceCode||' - '||COALESCE((SELECT c.Description FROM code c WHERE c.Category='Nace'||x.NaceVersion AND c.Code=x.NaceCode AND c.Language='NL' LIMIT 1),'omschrijving onbekend')||' ('||x.Classification||')') FROM activity x WHERE x.EntityNumber=e.EnterpriseNumber),e.TypeOfEnterprise,e.JuridicalForm,e.Status", "SELECT COUNT(*)", 1);
    let totaal:i64=connection.query_row(&count_sql.replacen(" LIMIT 100","",1),params_from_iter(values.iter()),|r|r.get(0)).map_err(|e|e.to_string())?;
    Ok(SearchResult{totaal,rijen:read_companies(&connection,&sql,&values)?})
}

#[tauri::command]
fn export_csv(request: SearchRequest) -> Result<String,String> {
    let connection=Connection::open(resolved_db_path(&request.db_path)).map_err(|e|e.to_string())?; let (sql,values)=query_parts(&request,None); let rows=read_companies(&connection,&sql,&values)?;
    let mut out=String::from("KBO-nummer,Startdatum,Naam,Adres,Contacten,Activiteiten,Type,Rechtsvorm,Status\n");
    for r in rows { let vals=[r.kbo_nummer,r.startdatum,r.naam,r.adres,r.contacten,r.activiteiten,r.ondernemings_type,r.rechtsvorm,r.status]; out.push_str(&vals.map(|v|format!("\"{}\"",v.replace('"',"\"\""))).join(",")); out.push('\n'); }
    Ok(out)
}

#[tauri::command]
fn import_kbo_zip(zip_path:String)->Result<String,String> {
    let source=Path::new(&zip_path); let parent=source.parent().unwrap_or(Path::new(".")); let stem=source.file_stem().and_then(|s|s.to_str()).unwrap_or("kbo_open_data"); let output:PathBuf=parent.join(format!("{stem}.sqlite"));
    let file=File::open(source).map_err(|e|format!("Zip openen mislukt: {e}"))?; let mut archive=ZipArchive::new(file).map_err(|e|e.to_string())?; let mut connection=Connection::open(&output).map_err(|e|e.to_string())?;
    connection.execute_batch("PRAGMA journal_mode=OFF; PRAGMA synchronous=OFF; PRAGMA temp_store=MEMORY;").map_err(|e|e.to_string())?;
    for name in ["enterprise.csv","establishment.csv","denomination.csv","address.csv","contact.csv","activity.csv","branch.csv","meta.csv","code.csv"] { let mut entry=archive.by_name(name).map_err(|e|format!("{name}: {e}"))?; let mut reader=ReaderBuilder::new().has_headers(true).flexible(true).from_reader(&mut entry); let headers=reader.headers().map_err(|e|e.to_string())?.clone(); let table=name.trim_end_matches(".csv"); let cols=headers.iter().map(|h|format!("\"{}\" TEXT",h.replace('"',"\"\""))).collect::<Vec<_>>().join(","); connection.execute(&format!("DROP TABLE IF EXISTS \"{table}\""),[]).map_err(|e|e.to_string())?; connection.execute(&format!("CREATE TABLE \"{table}\" ({cols})"),[]).map_err(|e|e.to_string())?; let marks=(0..headers.len()).map(|_|"?").collect::<Vec<_>>().join(","); let insert=format!("INSERT INTO \"{table}\" VALUES ({marks})"); let tx=connection.transaction().map_err(|e|e.to_string())?; for record in reader.records() { let record=record.map_err(|e|e.to_string())?; let vals=record.iter().map(|v|v.to_string()).collect::<Vec<_>>(); tx.execute(&insert,params_from_iter(vals.iter())).map_err(|e|e.to_string())?; } tx.commit().map_err(|e|e.to_string())?; }
    Ok(output.to_string_lossy().to_string())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run(){tauri::Builder::default().plugin(tauri_plugin_opener::init()).plugin(tauri_plugin_dialog::init()).invoke_handler(tauri::generate_handler![search_companies,export_csv,import_kbo_zip]).run(tauri::generate_context!()).expect("error while running tauri application");}
