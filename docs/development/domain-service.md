# Domain Services: Complete Role Summary

## Core Definition

Domain Services encapsulate **pure business logic** that doesn't naturally belong to any single Entity or Value Object. They represent business operations and rules that domain experts explicitly talk about.

## Primary Role

**Stateless business logic operations** that:

- Execute complex business calculations
- Implement business rules and policies
- Coordinate between multiple aggregates
- Provide domain algorithms and formulas

## Key Characteristics

### 1. **Stateless & Pure**

- No internal state between calls
- Deterministic: same inputs = same outputs
- Often implementable as pure functions
- No side effects

### 2. **Domain Layer Resident**

- Lives in the core domain layer
- No infrastructure dependencies
- Speaks only in ubiquitous language
- Represents actual business concepts

### 3. **Not Entities or Value Objects**

- Operations that span multiple aggregates
- Business logic without a natural "home"
- Verbs in your domain language (calculate, validate, assess)

## What Domain Services ARE

✅ **Business Operations**

- `PricingService.calculateDiscount()`
- `RiskAssessmentService.evaluateRisk()`
- `TaxCalculationService.computeTax()`

✅ **Domain Algorithms**

- Complex calculations from business rules
- Business policy implementations
- Domain-specific validations

✅ **Pure Functions**

- Input → Process → Output
- No database calls
- No external API calls
- No state management

## What Domain Services ARE NOT

❌ **Not Helpers or Utilities**

- Not generic string/date manipulation
- Not technical conveniences
- Must represent business concepts

❌ **Not Repositories**

- Don't fetch or store data
- Don't manage persistence
- Don't handle database connections

❌ **Not Use Cases**

- Don't orchestrate entire workflows
- Don't handle user authorization
- Don't manage transactions
- Don't coordinate infrastructure

## Relationship with Other Components

### Used By Use Cases

```rust
pub struct ProcessOrderUseCase {
    repository: Box<dyn OrderRepository>,        // For data access
    pricing_service: Box<dyn PricingService>,    // For business logic
    tax_service: Box<dyn TaxService>,           // For business logic
}
```

### Working With Entities

```rust
// Domain Service processes entities but doesn't own them
let discount = discount_service.calculate(&customer, &order);
let updated_order = order.apply_discount(discount);
```

## Implementation Patterns

### Simplest: Pure Function Module

```rust
pub mod interest_calculation {
    pub fn calculate(principal: Money, rate: f64, time: Duration) -> Money {
        principal * rate * time.as_years()
    }
}
```

### Common: Stateless Struct

```rust
pub struct TaxService;
impl TaxService {
    pub fn calculate(&self, amount: Money, location: &Location) -> Money {
        // Pure calculation based on inputs
    }
}
```

### Complex: Composing Other Domain Services

```rust
pub struct PricingService {
    tax_service: Box<dyn TaxService>,
    discount_service: Box<dyn DiscountService>,
}
```

## The Litmus Test

Ask yourself:

1. **"Is this a business concept that domain experts discuss?"**

   - ✅ "Calculate shipping costs" → Domain Service
   - ❌ "Format date string" → Utility

2. **"Is this pure business logic without infrastructure needs?"**

   - ✅ "Compute compound interest" → Domain Service
   - ❌ "Send email notification" → Infrastructure Service

3. **"Does this operation span multiple aggregates or have no natural home?"**
   - ✅ "Transfer funds between accounts" → Domain Service
   - ❌ "Update user password" → Entity method

## Why They Matter

### Business Logic Isolation

- All critical business rules in one place
- Independent of infrastructure changes
- Easy to understand and modify

### Testability

- Simple unit tests without mocks
- No database or network required
- Fast, deterministic tests

### Reusability

- Same service used by multiple use cases
- Consistent business logic application
- Single source of truth for rules

## In Summary

Domain Services are **pure business logic functions** that embody core domain knowledge. They're not helpers, not repositories, and not use cases. They're the **mathematical formulas of your business domain** - stateless, pure, and focused solely on implementing the complex business rules that make your domain unique.

They answer: **"How does this business calculation/rule/operation work?"**

Not: "Where is the data?" (Repository) or "What does the user want to do?" (Use Case)
