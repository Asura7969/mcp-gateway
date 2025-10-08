#[cfg(test)]
mod tests {
    use serde::{Deserialize, Serialize};
    use surrealdb::engine::local::Mem;
    use surrealdb::RecordId;
    use surrealdb::Surreal;

    #[derive(Debug, Serialize)]
    struct Name<'a> {
        first: &'a str,
        last: &'a str,
    }

    #[derive(Debug, Serialize)]
    struct Person<'a> {
        title: &'a str,
        name: Name<'a>,
        marketing: bool,
    }

    #[derive(Debug, Serialize)]
    struct Responsibility {
        marketing: bool,
    }

    #[derive(Debug, Deserialize)]
    struct Record {
        #[allow(dead_code)]
        id: RecordId,
    }

    #[derive(Serialize, Deserialize, Debug)]
    struct PersonV2<'a> {
        name: &'a str,
        age: u8,
    }

    #[derive(Serialize, Deserialize, Debug)]
    struct Organization<'a> {
        name: &'a str,
    }

    #[derive(Debug, Deserialize)]
    struct MemberRecord {
        name: String,
        age: u8,
    }

    // 也可以查询特定人物所属的组织
    #[derive(Debug, Deserialize)]
    struct OrgRecord {
        name: String,
    }

    // 向量搜索相关的数据结构
    #[derive(Serialize, Deserialize, Debug)]
    struct Document {
        content: String,
        embedding: Vec<f64>,
    }

    #[derive(Debug, Deserialize)]
    struct DocumentRecord {
        id: RecordId,
        content: String,
        embedding: Vec<f64>,
    }

    #[derive(Debug, Deserialize)]
    struct SearchResult {
        id: RecordId,
        content: String,
        embedding: Vec<f64>,
        distance: f64,
    }

    #[tokio::test]
    async fn test_graph() -> Result<(), Box<dyn std::error::Error>> {
        // 连接到内存数据库
        let db = Surreal::new::<Mem>(()).await?;

        // 选择命名空间和数据库
        db.use_ns("test").use_db("test").await?;

        // insert
        let alice: Record = db
            .create("person")
            .content(PersonV2 {
                name: "Alice",
                age: 30,
            })
            .await?
            .unwrap();

        let bob: Record = db
            .create("person")
            .content(PersonV2 {
                name: "Bob",
                age: 25,
            })
            .await?
            .unwrap();
        //
        let org: Record = db
            .create("organization")
            .content(Organization {
                name: "SurrealDB Inc.",
            })
            .await?
            .unwrap();

        // 建立关系：Alice 属于 该组织
        // 使用 RELATE语句创建边，表名为 member_of
        let r: Option<Record> = db
            .query("RELATE $person->member_of->$org")
            .bind(("person", alice.id.clone()))
            .bind(("org", org.id.clone()))
            .await?
            .take(0)?;

        dbg!(&r);

        // 图检索 - 查找属于特定组织的所有人员
        let members: Vec<MemberRecord> = db
            .query(
                "
        SELECT
            name,
            age
        FROM (SELECT VALUE in FROM member_of WHERE out.name = 'SurrealDB Inc.')
    ",
            )
            .await?
            .take(0)?;
        dbg!(members);

        let orgs: Vec<OrgRecord> = db
            .query(
                "
        SELECT
            name
        FROM
            $person->member_of->organization
    ",
            )
            // 使用之前创建的Alice的记录ID
            .bind(("person", alice.id))
            .await?
            .take(0)?;
        dbg!(orgs);

        // 更新Bob的年龄为26岁
        let new_bob: Record = db
            .update(&bob.id)
            .content(PersonV2 {
                name: "Bob",
                age: 26,
            })
            .await?
            .unwrap();
        dbg!(&new_bob);

        let r: Option<Record> = db
            .query("RELATE $person->member_of->$org")
            .bind(("person", new_bob.id.clone()))
            .bind(("org", org.id))
            .await?
            .take(0)?;

        dbg!(&r);

        // 创建新组织 "New Co."
        let new_org: Record = db
            .create("organization")
            .content(Organization { name: "New Co." })
            .await?
            .unwrap();
        dbg!("新创建的组织:", &new_org);

        // 创建 Bob 到 "New Co." 的关系
        let bob_to_new_org: Option<Record> = db
            .query("RELATE $person->member_of->$org")
            .bind(("person", new_bob.id.clone()))
            .bind(("org", new_org.id))
            .await?
            .take(0)?;
        dbg!("Bob到New Co.的关系:", &bob_to_new_org);

        // 查询新创建的关系 - 查找属于 "New Co." 的所有成员
        let new_org_members: Vec<MemberRecord> = db
            .query(
                "
            SELECT
                name,
                age
            FROM (SELECT VALUE in FROM member_of WHERE out.name = 'New Co.')
        ",
            )
            .await?
            .take(0)?;
        dbg!("New Co. 的成员:", &new_org_members);

        // 查询所有现存的关系 - 使用简化的查询
        let all_relations_count: Vec<serde_json::Value> = db
            .query("SELECT count() FROM member_of GROUP ALL")
            .await?
            .take(0)?;
        dbg!("member_of关系总数:", &all_relations_count);

        // 查询所有组织的成员情况
        let surrealdb_members: Vec<MemberRecord> = db
            .query(
                "
            SELECT
                name,
                age
            FROM (SELECT VALUE in FROM member_of WHERE out.name = 'SurrealDB Inc.')
        ",
            )
            .await?
            .take(0)?;
        dbg!("SurrealDB Inc. 的所有成员:", &surrealdb_members);

        let newco_members: Vec<MemberRecord> = db
            .query(
                "
            SELECT
                name,
                age
            FROM (SELECT VALUE in FROM member_of WHERE out.name = 'New Co.')
        ",
            )
            .await?
            .take(0)?;
        dbg!("New Co. 的所有成员:", &newco_members);

        Ok(())
    }

    #[tokio::test]
    async fn test_crud() -> surrealdb::Result<()> {
        // Create database connection in memory
        let db = Surreal::new::<Mem>(()).await?;

        // Create database connection using RocksDB
        // let db = Surreal::new::<RocksDb>("path/to/database-folder").await?;

        // Select a specific namespace / database
        db.use_ns("test").use_db("test").await?;

        // Create a new person with a random id
        let created: Option<Record> = db
            .create("person")
            .content(Person {
                title: "Founder & CEO",
                name: Name {
                    first: "Tobie",
                    last: "Morgan Hitchcock",
                },
                marketing: true,
            })
            .await?;
        dbg!(created);

        // Update a person record with a specific id
        let updated: Option<Record> = db
            .update(("person", "jaime"))
            .merge(Responsibility { marketing: true })
            .await?;
        dbg!(updated);

        // Select all people records
        let people: Vec<Record> = db.select("person").await?;
        dbg!(people);

        // Perform a custom advanced query
        let groups = db
            .query("SELECT marketing, count() FROM type::table($table) GROUP BY marketing")
            .bind(("table", "person"))
            .await?;
        dbg!(groups);
        Ok(())
    }

    #[tokio::test]
    async fn test_vector_search() -> Result<(), Box<dyn std::error::Error>> {
        // 连接到内存数据库
        let db = Surreal::new::<Mem>(()).await?;

        // 选择命名空间和数据库
        db.use_ns("test").use_db("vector_test").await?;

        // 创建文档表的向量索引
        // 使用余弦相似度作为距离度量
        let index_result: Vec<serde_json::Value> = db
            .query("DEFINE INDEX embedding_idx ON TABLE document FIELDS embedding MTREE DIMENSION 4 DIST COSINE")
            .await?
            .take(0)?;
        dbg!("创建向量索引结果:", &index_result);

        // 插入带有向量嵌入的文档数据
        let doc1: DocumentRecord = db
            .create("document")
            .content(Document {
                content: "苹果是一种水果".to_string(),
                embedding: vec![0.1, 0.2, 0.3, 0.4],
            })
            .await?
            .unwrap();
        dbg!("插入文档1:", &doc1);

        let doc2: DocumentRecord = db
            .create("document")
            .content(Document {
                content: "香蕉也是水果".to_string(),
                embedding: vec![0.15, 0.25, 0.35, 0.45],
            })
            .await?
            .unwrap();
        dbg!("插入文档2:", &doc2);

        let doc3: DocumentRecord = db
            .create("document")
            .content(Document {
                content: "汽车是交通工具".to_string(),
                embedding: vec![0.8, 0.7, 0.6, 0.5],
            })
            .await?
            .unwrap();
        dbg!("插入文档3:", &doc3);

        let doc4: DocumentRecord = db
            .create("document")
            .content(Document {
                content: "橙子富含维生素C".to_string(),
                embedding: vec![0.12, 0.22, 0.32, 0.42],
            })
            .await?
            .unwrap();
        dbg!("插入文档4:", &doc4);

        // 查询向量 - 寻找与"水果"相关的内容
        let query_vector = vec![0.11, 0.21, 0.31, 0.41];

        // 执行向量相似性搜索 - 查找最相似的3个文档
        let search_results: Vec<SearchResult> = db
            .query(
                "
                SELECT 
                    id,
                    content,
                    embedding,
                    vector::similarity::cosine(embedding, $query_vector) AS distance
                FROM document 
                WHERE embedding <|3|> $query_vector
                ORDER BY distance DESC
            ",
            )
            .bind(("query_vector", query_vector.clone()))
            .await?
            .take(0)?;

        dbg!("查询向量:", &query_vector);
        dbg!("向量搜索结果 (前3个最相似):", &search_results);

        // 验证搜索结果
        assert!(!search_results.is_empty(), "搜索结果不应为空");
        assert!(search_results.len() <= 3, "搜索结果应不超过3个");

        // 验证结果按相似度排序（相似度越大越相似）
        for i in 1..search_results.len() {
            assert!(
                search_results[i - 1].distance >= search_results[i].distance,
                "搜索结果应按相似度降序排列"
            );
        }

        // 执行另一个向量搜索 - 寻找与"交通"相关的内容
        let transport_query = vec![0.75, 0.65, 0.55, 0.45];

        let transport_results: Vec<SearchResult> = db
            .query(
                "
                SELECT 
                    id,
                    content,
                    embedding,
                    vector::similarity::cosine(embedding, $query_vector) AS distance
                FROM document 
                WHERE embedding <|2|> $query_vector
                ORDER BY distance DESC
            ",
            )
            .bind(("query_vector", transport_query.clone()))
            .await?
            .take(0)?;

        dbg!("交通相关查询向量:", &transport_query);
        dbg!("交通相关搜索结果:", &transport_results);

        // 查询所有文档以验证数据完整性
        let all_docs: Vec<DocumentRecord> = db.select("document").await?;
        dbg!("所有文档:", &all_docs);
        assert_eq!(all_docs.len(), 4, "应该有4个文档");

        println!("✅ 向量搜索测试完成！");
        Ok(())
    }
}
