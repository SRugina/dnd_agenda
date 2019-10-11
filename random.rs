fn main() {
    let orgs = vec![1, 3, 14, 12];

    let events: Vec<_> = orgs.iter().map(|org_id| vec![org_id, &2]).collect();

    println!("hi");
    println!("{:#?}", events);
}
