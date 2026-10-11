// CR 603.2/603.3d and pinned Viashino Pyromancer: casting has no target;
// resolution puts the creature on the battlefield before its trigger targets.
#[test]
fn full_pool_triggers_played_etb_choice() {
    let mut doc: Value = serde_json::from_str(include_str!(
        "../../../fixtures/reference/full-pool-spells.json"
    )).unwrap();
    let mut case = doc["cases"][0].clone();
    let order = case["chance"][0]["after"].as_array_mut().unwrap();
    let a = order.iter().position(|v| v == "0/dragon-fodder/0").unwrap();
    let b = order.iter().position(|v| v == "0/viashino-pyromancer/0").unwrap();
    order.swap(a,b);
    case["play"][39]["source"] = json!("0/viashino-pyromancer/0");
    let mut target = case["play"][46].clone();
    target["sequence"] = json!(47);
    target["actor"] = json!(0);
    target["kind"] = json!("order_trigger");
    case["play"].as_array_mut().unwrap().push(target);
    doc["cases"] = json!([case]);
    let result = spells_consume(&mut Game::new().unwrap(), doc, &mut Value::Null);
    assert!(result.is_ok(), "legal played Pyromancer ETB must accept its post-resolution trigger choice: {result:?}");
}
