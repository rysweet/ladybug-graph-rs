/// Fluent builder for constructing Cypher query strings.
///
/// Supports MATCH, CREATE, RETURN, WHERE, SET, DELETE, ORDER BY, LIMIT, SKIP.
#[derive(Debug, Clone, Default)]
pub struct CypherBuilder {
    match_clauses: Vec<String>,
    create_clauses: Vec<String>,
    where_clauses: Vec<String>,
    set_clauses: Vec<String>,
    delete_clauses: Vec<String>,
    return_clause: Option<String>,
    order_by: Option<String>,
    limit: Option<u64>,
    skip: Option<u64>,
    with_clauses: Vec<String>,
    merge_clauses: Vec<String>,
    raw_prefix: Vec<String>,
    raw_suffix: Vec<String>,
}

impl CypherBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a MATCH clause.
    pub fn match_pattern(mut self, pattern: &str) -> Self {
        self.match_clauses.push(pattern.to_string());
        self
    }

    /// Add a CREATE clause.
    pub fn create(mut self, pattern: &str) -> Self {
        self.create_clauses.push(pattern.to_string());
        self
    }

    /// Add a MERGE clause.
    pub fn merge(mut self, pattern: &str) -> Self {
        self.merge_clauses.push(pattern.to_string());
        self
    }

    /// Add a WHERE condition (conditions are ANDed).
    pub fn where_clause(mut self, condition: &str) -> Self {
        self.where_clauses.push(condition.to_string());
        self
    }

    /// Add a SET expression.
    pub fn set(mut self, expr: &str) -> Self {
        self.set_clauses.push(expr.to_string());
        self
    }

    /// Add a DELETE expression.
    pub fn delete(mut self, expr: &str) -> Self {
        self.delete_clauses.push(expr.to_string());
        self
    }

    /// Set the RETURN clause.
    pub fn return_expr(mut self, expr: &str) -> Self {
        self.return_clause = Some(expr.to_string());
        self
    }

    /// Set ORDER BY.
    pub fn order_by(mut self, expr: &str) -> Self {
        self.order_by = Some(expr.to_string());
        self
    }

    /// Set LIMIT.
    pub fn limit(mut self, n: u64) -> Self {
        self.limit = Some(n);
        self
    }

    /// Set SKIP.
    pub fn skip(mut self, n: u64) -> Self {
        self.skip = Some(n);
        self
    }

    /// Add a WITH clause.
    pub fn with(mut self, expr: &str) -> Self {
        self.with_clauses.push(expr.to_string());
        self
    }

    /// Add raw Cypher before the main query.
    pub fn raw_prefix(mut self, cypher: &str) -> Self {
        self.raw_prefix.push(cypher.to_string());
        self
    }

    /// Add raw Cypher after the main query.
    pub fn raw_suffix(mut self, cypher: &str) -> Self {
        self.raw_suffix.push(cypher.to_string());
        self
    }

    /// Build the final Cypher query string.
    pub fn build(&self) -> String {
        let mut parts: Vec<String> = Vec::new();

        for raw in &self.raw_prefix {
            parts.push(raw.clone());
        }

        for m in &self.match_clauses {
            parts.push(format!("MATCH {m}"));
        }

        if !self.where_clauses.is_empty() {
            parts.push(format!("WHERE {}", self.where_clauses.join(" AND ")));
        }

        for w in &self.with_clauses {
            parts.push(format!("WITH {w}"));
        }

        for c in &self.create_clauses {
            parts.push(format!("CREATE {c}"));
        }

        for m in &self.merge_clauses {
            parts.push(format!("MERGE {m}"));
        }

        for s in &self.set_clauses {
            parts.push(format!("SET {s}"));
        }

        for d in &self.delete_clauses {
            parts.push(format!("DELETE {d}"));
        }

        if let Some(ref ret) = self.return_clause {
            parts.push(format!("RETURN {ret}"));
        }

        if let Some(ref ob) = self.order_by {
            parts.push(format!("ORDER BY {ob}"));
        }

        if let Some(s) = self.skip {
            parts.push(format!("SKIP {s}"));
        }

        if let Some(l) = self.limit {
            parts.push(format!("LIMIT {l}"));
        }

        for raw in &self.raw_suffix {
            parts.push(raw.clone());
        }

        parts.join(" ")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn simple_match_return() {
        let q = CypherBuilder::new()
            .match_pattern("(n:Person)")
            .return_expr("n")
            .build();
        assert_eq!(q, "MATCH (n:Person) RETURN n");
    }

    #[test]
    fn match_with_where() {
        let q = CypherBuilder::new()
            .match_pattern("(n:Person)")
            .where_clause("n.age > 20")
            .return_expr("n")
            .build();
        assert_eq!(q, "MATCH (n:Person) WHERE n.age > 20 RETURN n");
    }

    #[test]
    fn match_with_multiple_where() {
        let q = CypherBuilder::new()
            .match_pattern("(n:Person)")
            .where_clause("n.age > 20")
            .where_clause("n.name = 'Alice'")
            .return_expr("n")
            .build();
        assert_eq!(
            q,
            "MATCH (n:Person) WHERE n.age > 20 AND n.name = 'Alice' RETURN n"
        );
    }

    #[test]
    fn create_node() {
        let q = CypherBuilder::new()
            .create("(n:Person {name: 'Alice'})")
            .return_expr("n")
            .build();
        assert_eq!(q, "CREATE (n:Person {name: 'Alice'}) RETURN n");
    }

    #[test]
    fn match_set() {
        let q = CypherBuilder::new()
            .match_pattern("(n:Person)")
            .where_clause("n.name = 'Alice'")
            .set("n.age = 30")
            .build();
        assert_eq!(q, "MATCH (n:Person) WHERE n.name = 'Alice' SET n.age = 30");
    }

    #[test]
    fn match_delete() {
        let q = CypherBuilder::new()
            .match_pattern("(n:Person)")
            .where_clause("n.name = 'Alice'")
            .delete("n")
            .build();
        assert_eq!(q, "MATCH (n:Person) WHERE n.name = 'Alice' DELETE n");
    }

    #[test]
    fn order_by_limit_skip() {
        let q = CypherBuilder::new()
            .match_pattern("(n:Person)")
            .return_expr("n")
            .order_by("n.name")
            .skip(5)
            .limit(10)
            .build();
        assert_eq!(
            q,
            "MATCH (n:Person) RETURN n ORDER BY n.name SKIP 5 LIMIT 10"
        );
    }

    #[test]
    fn merge_clause() {
        let q = CypherBuilder::new()
            .merge("(n:Person {name: 'Alice'})")
            .return_expr("n")
            .build();
        assert_eq!(q, "MERGE (n:Person {name: 'Alice'}) RETURN n");
    }

    #[test]
    fn with_clause() {
        let q = CypherBuilder::new()
            .match_pattern("(n:Person)")
            .with("n, n.age AS age")
            .return_expr("age")
            .build();
        assert_eq!(q, "MATCH (n:Person) WITH n, n.age AS age RETURN age");
    }

    #[test]
    fn raw_prefix_and_suffix() {
        let q = CypherBuilder::new()
            .raw_prefix("// comment")
            .match_pattern("(n:Person)")
            .return_expr("n")
            .raw_suffix(";")
            .build();
        assert_eq!(q, "// comment MATCH (n:Person) RETURN n ;");
    }

    #[test]
    fn empty_builder() {
        let q = CypherBuilder::new().build();
        assert_eq!(q, "");
    }

    #[test]
    fn builder_clone() {
        let b = CypherBuilder::new().match_pattern("(n)").return_expr("n");
        let b2 = b.clone();
        assert_eq!(b.build(), b2.build());
    }

    #[test]
    fn multiple_match_clauses() {
        let q = CypherBuilder::new()
            .match_pattern("(a:Person)")
            .match_pattern("(b:City)")
            .return_expr("a, b")
            .build();
        assert_eq!(q, "MATCH (a:Person) MATCH (b:City) RETURN a, b");
    }

    #[test]
    fn multiple_create_clauses() {
        let q = CypherBuilder::new()
            .create("(a:Person {name: 'A'})")
            .create("(b:Person {name: 'B'})")
            .build();
        assert_eq!(
            q,
            "CREATE (a:Person {name: 'A'}) CREATE (b:Person {name: 'B'})"
        );
    }
}
