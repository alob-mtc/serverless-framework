migrate:
	sea-orm-cli migrate up -n $(n)
migrate-down:
	sea-orm-cli migrate down -n $(n)
create-migration:
	sea-orm-cli migrate generate $(name)
generate-entity:
	sea-orm-cli generate entity -o db_entities/src