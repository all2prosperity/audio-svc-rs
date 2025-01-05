export DATABASE_URL="postgres://ckz:1@10.211.55.8:5432/llm"
diesel migration generate --diff-schema create_posts
# diesel migration generate create_table_name
# diesel migration run