mod checker;

fn parse_combo_list(path: &str) -> Result<Vec<checker::Combo>, Box<dyn std::error::Error>> {
    let mut combos = Vec::new();
    let combo_list = std::fs::read_to_string(path)?;
    for line in combo_list.lines() {
        let combo = line.trim().split(":").collect::<Vec<&str>>();
        if combo.len() != 2 {
            continue;
        }
        combos.push(checker::Combo {
            username: combo[0].to_string(),
            password: combo[1].to_string(),
        });
    }
    Ok(combos)
}

#[tokio::main]
async fn main() {
    let path_to_file: Vec<String> = std::env::args().collect();
    for combo in parse_combo_list(&path_to_file[1]).expect("Invalid path to file") {
        let checker_client = checker::Checker::new();
        let res = checker_client.check_combo(&combo).await.unwrap();
        println!("{:?} - {:?}", combo, res);
    }
}
