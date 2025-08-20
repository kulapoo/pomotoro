# Domain Entity

```rust
use crate::prelude::*;

#[derive(Debug, Clone)]
pub struct Entity {
    id: EntityId,
}

impl Entity {
    pub fn new(/* params */) -> Result<Self, Error> {
        Ok(Self { /* fields */ })
    }

    pub fn business_operation(&mut self) -> Result<Event, Error> {
        // Pure business logic only
    }
}
```

# Repository Trait

```rust
#[async_trait]
pub trait Repository: Send + Sync {
    async fn find(&self, id: Id) -> Result<Entity, Error>;
    async fn save(&self, entity: &Entity) -> Result<(), Error>;
}
```

# Use Case

```rust
use domain::prelude::*;

pub async fn use_case_name(
    repo: &impl Repository,
    service: &impl DomainService,
    cmd: Command,
) -> Result<Response, Error> {
    let aggregate = repo.find(cmd.id).await?;
    let events = aggregate.operation()?;
    repo.save(&aggregate).await?;

    for event in events {
        service.handle(event).await?;
    }

    Ok(Response::new())
}
```

# Infrastructure Repository

```rust
use domain::prelude::*;

pub struct PostgresRepository {
    pool: PgPool,
}

#[async_trait]
impl Repository for PostgresRepository {
    async fn find(&self, id: Id) -> Result<Entity, Error> {
        sqlx::query_as!(/* ... */)
            .fetch_one(&self.pool)
            .await
            .map_err(Into::into)
    }
}
```