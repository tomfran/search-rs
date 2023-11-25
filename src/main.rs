mod trie;
use trie::Trie;

fn main() {
    let mut t = Trie::new();

    t.insert("hello", 1);
    t.insert("hell", 2);
    t.insert("hey", 3);

    println!("{:?}", t.get("hello"));
    println!("{:?}", t.get("hell"));
    println!("{:?}", t.get("hey"));
    println!("{:?}", t.get("he"));

    println!("{:?}", t.get_by_prefix("hel"));
}
