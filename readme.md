Video Explanation (https://drive.google.com/file/d/1z-5PB06MzU4GxiGXjFw59rKCwH78sEOd/view?usp=sharing)
{ Governance Budget Allocator }

A Rust smart contract for managing shared budgets with role-based permissions on Stellar/Soroban.


Contract Summary

This contract solves a common DAO problem:
how  to let multiple people manage a shared treasury without giving anyone unlimited power?

The solution uses (two-tier permission):

Owner

-Decides who can manage the budget
-Adds and removes operators

Operators

-Can increase or decrease the budget
-Must stay within predefined minimum and maximum limits

Real-world use case:
A DAO where the governance council appoints budget managers who can allocate funds for initiatives, but cannot drain the treasury or overspend beyond approved limits.

Why this design:
Separating access control (owner) from budget operations (operators) prevents any single person from having absolute power.
The min/max bounds act as safety rails that cannot be bypassed.

---

How to Build & Test

Build

```bash
cargo build --target wasm32-unknown-unknown --release
```

-Output:

```
target/wasm32-unknown-unknown/release/governance_budget.wasm
```

---

-Test:

```bash
cargo test
```

Expected result:(checked in the system #)
All tests passing.



 State & Flows (Initialization and Important Transactions)

 Initial Setup

1. Deploy the contract
2. Call
   `initialize(owner_address, initial_amount, min_limit, max_limit)`
3. Contract validates:
   `min ≤ initial ≤ max`
4. Stores:

   -owner
   - empty operators list
   - budget state

---

 Adding Budget Managers:

Owner calls:

```
add_operator(manager_address)
```

Checks performed:

- Is the caller the owner?
-Is this address already an operator?

If valid, the address is added to the operators list.

---
 Adjusting the Budget

Increasing the budget

Operator calls:

```
increase_budget(500)
```

Steps:

-Authenticate: does the signature match the caller?
-Authorize: is the caller an operator?
-Validate: `current + 500 ≤ max`
-Update budget and return the new value

---

Decreasing the budget

Operator calls:

```
decrease_budget(200)
```

Steps:

- Same authentication and authorization checks
- Validate: `current - 200 ≥ min`
- Update budget and return the new value

---

 Error Handling (What Could Go Wrong)

The contract returns (specific errors instead of failing silently):

- Non-owner tries to add an operator → `NotOwner`
- Non-operator tries to modify the budget → `NotOperator`
- Increase exceeds maximum → `ExceedsMax`
- Decrease goes below minimum → `BelowMin`
- Arithmetic overflow → `Overflow` (caught by `checked_add`)
- Arithmetic underflow → `Underflow` (caught by `checked_sub`)

This makes failures predictable and easy to debug.

---

Specify Known Limitations

- No audit trail — historical changes are not stored, only the current state
- Single owner — no ownership transfer or multisig support
- Immutable limits — min/max bounds are fixed at initialization
- No time delays — budget changes happen instantly (no voting or cooldown)
- No events — off-chain systems must poll state to detect changes
- Linear operator lookup — operator checks are O(n), fine for small groups

---

 Deployment

Network:= Stellar Testnet

-Contract ID:
  `CDHW3VVPAXQBKQQN7RX6QXBVGE7SCRSEHCCQY2PDCQB7YRWPIMWS3ITR`

-Explorer:
  [https://stellar.expert/explorer/testnet/contract/CDHW3VVPAXQBKQQN7RX6QXBVGE7SCRSEHCCQY2PDCQB7YRWPIMWS3ITR](https://stellar.expert/explorer/testnet/contract/CDHW3VVPAXQBKQQN7RX6QXBVGE7SCRSEHCCQY2PDCQB7YRWPIMWS3ITR)

Gas & Compute Considerations:
-Storage Costs
-The contract stores:
-Owner address (one storage slot)
-Operators list (one entry per operator)
-Budget state (current, min, max)
-Even with around 10 operators, total storage usage stays under 1 KB, which is very small.

How Fast Things Run
-get_budget: Instant — simple storage read
-add_operator: Slows down as operators increase (checks for duplicates)
-increase_budget / decrease_budget: Operator authorization requires scanning the list
-In computer science terms, these operations are O(n) — they scale linearly with the number of operators.


How to Make It Faster for Production
-Right now, checking “is this person an operator?” requires scanning the whole list.
-That’s like checking if a name exists by reading every page of a phone book.
-Better approaches for large DAOs:
-Use a Set instead of a list for instant lookups
-Store operator count separately
-Add pagination for very large operator lists

Performance Summary
-Each function call uses roughly (10,000 compute instructions)
-Complex DeFi swaps often exceed ((100,000+ instructions))
-This contract is lightweight and efficient
-The current design works very well for DAOs with fewer than ~50 operators.
-For very large organizations, data structures can be optimized further.



Short Takeaway
-This contract focuses on (clarity, safety, and correctness) over complexity.
-It demonstrates how simple design choices can enforce strong governance rules on-chain.

