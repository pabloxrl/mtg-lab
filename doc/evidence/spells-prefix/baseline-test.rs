// GH-272 behavioral baseline: a reachable sorcery must enter its real cast
// continuation. Dragon Fodder costs 1R (pinned Oracle); turn-three main has
// two untapped Mountains and the explicitly shuffled physical card in hand.
#[test]
fn full_pool_spells_literal_checkpoints() {
    let mut doc = priority_fixture();
    doc["cases"].as_array_mut().unwrap().truncate(1);
    let case = &mut doc["cases"][0];
    let old = "0/swab-goblin/0";
    let new = "0/dragon-fodder/0";
    for id in case["chance"][0]["after"].as_array_mut().unwrap() {
        if id == old { *id = json!(new); }
        else if id == new { *id = json!(old); }
    }
    let play = case["play"].as_array_mut().unwrap();
    let cast = play.iter().position(|e| e["kind"] == "cast").unwrap();
    assert_eq!(play[cast]["source"], old);
    play[cast]["source"] = json!(new);
    play.truncate(cast + 1);
    let (opening, reset) = priority_projection(&doc).unwrap();
    let mut game = Game::new().unwrap();
    let result = london_continue(&mut game, &reset.cases[0], &opening["cases"][0], |g, tape| {
        let mut client = PlayedClient::new(g, tape, &doc["cases"][0])?;
        let play: Vec<PlayedChoice> = serde_json::from_value(doc["cases"][0]["play"].clone()).unwrap();
        for e in play { client.apply(g, &e)?; }
        require(g.turns.payment.is_some(), "/Fodder payment continuation missing")?;
        Ok(())
    });
    assert!(result.is_ok(), "legal normal-reset Dragon Fodder cast must reach payment: {:?}", result.err());
}
