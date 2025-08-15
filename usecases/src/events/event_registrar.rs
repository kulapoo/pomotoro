use super::HandlerRegistry;
use domain::Result;

pub trait EventRegistrar: Send + Sync {
    fn register(&self, registry: &mut HandlerRegistry) -> Result<()>;
    fn unregister(&self, registry: &mut HandlerRegistry) -> Result<()>;
}