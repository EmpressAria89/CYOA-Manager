#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn persistence_retains_old_aliases_and_rejects_conflicts() {
        let root = std::env::temp_dir().join(uuid::Uuid::new_v4().to_string());
        let path = root.join("author-aliases.json");
        let mut authors = read(&path).unwrap();
        authors[0].aliases.push("Gaston1597".into());
        validate(&mut authors).unwrap();
        write(&path, &authors).unwrap();
        let persisted = read(&path).unwrap();
        assert_eq!(persisted[0].aliases, ["Gaston1231", "Gaston1597"]);
        authors.push(AuthorAlias {
            id: uuid::Uuid::new_v4().to_string(),
            name: "Someone".into(),
            aliases: vec![" gaston1231 ".into()],
        });
        assert!(validate(&mut authors).is_err());
        assert_eq!(read(&path).unwrap(), persisted);
        std::fs::write(&path, b"broken").unwrap();
        assert!(read(&path).is_err());
        assert_eq!(std::fs::read(&path).unwrap(), b"broken");
        std::fs::remove_dir_all(root).unwrap();
    }
}
