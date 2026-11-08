#[test]
fn probe() {
    use velocity_frontend::string_manager::string_manager::string_crc;
    let want: [i64; 2] = [-38261373, 1065522401];
    for s in ["SaveGame_Yes", "SaveGame_YES", "SaveGame_Ok", "SaveGame_OK", "SaveGame_Continue", "SaveGame_New"] {
        println!("{} = {s}", string_crc(s) as i32);
    }
    let Ok(t) = std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/../../research/strings.tsv")) else { return };
    for l in t.lines() {
        let Some((_, s)) = l.split_once('\t') else { continue };
        let c = string_crc(s) as i32 as i64;
        if want.contains(&c) {
            println!("{c} = {s}");
        }
    }
}
