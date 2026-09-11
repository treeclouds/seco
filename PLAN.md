# SECO — Work Ledger

Canonical task tracking for the SECO marketplace (Rust / Loco / Axum / SeaORM). Repo: `treeclouds/seco`, branch `develop`. Reads bottom-up: oldest work first.

Status key: `[ ]` todo · `[~]` in progress · `[x]` done · `[!]` blocked

---

## Epic 1 — Security changeset (uncommitted, on `develop`)

- [~] Cart + order endpoints bind operations to authenticated user (IDOR) + return 404 (not leak)
- [~] `order_add` sets `final_price` server-side from `product.price`
- [~] Seller order lookup join fix: `o.seller_id` → `oi.seller_id`
- [~] `user_block` / `unblock` / `delete` enforce superuser (fix broken if/else-if)
- [~] `get_all_products` search filters (title/location/brand/category) → SeaORM bound params (SQLi fix)
- [~] Regression tests (requests layer): cart IDOR, self-product block, order IDOR, `final_price` server-side, superuser block/unblock/delete, seller-join fix, SQLi bound-param.
  - [!] Blocked: Tailscale SSH re-auth gate on VPS `100.101.60.102` (approve `https://login.tailscale.com/a/la59b3e730e420`, or add SSH key)
- [!] Fix orphaned test module declarations in `tests/requests/mod.rs` (6 `.rs` files never committed) — found by @qa-tester
- [ ] Final integration review of changeset → commit on `develop`

## Epic 2 — Config / secret audit (deploy hardening)

- [x] Rotate `JWT_SECRET` + `REFRESH_SECRET` in `/etc/seco/seco.env` (fresh 256-bit, 600 perms) → done on VPS
- [ ] Ensure `production.yaml` reads secrets from env, no committed defaults
- [ ] SMTP mailer: decide enable now vs later (verify/reset/magic-link email silent while commented out)

## Epic 3 — Clerk migration (optional, ~4.5–5 dev-days, pre-production window)

Backend scope (frontend `<ClerkProvider>` separate):
1. [ ] Replace HS256 JWT verifier with RS256 against Clerk JWKS (`https://<app>.clerk.accounts.dev`), check `iss` + `aud`
2. [ ] Identity mapping: `auth.claims.pid` → Clerk `sub`; swap `find_by_pid` → `find_or_create_by_clerk_sub` (call-site spread across controllers)
3. [ ] Rework `OptionalJwt` so valid Clerk token resolves user (guard the degrade-to-guest path)
4. [ ] Delete dead auth: register/login/logout/refresh/forgot/reset/magic-link + `password` hash col, email tokens, `refresh_tokens` entity, `REFRESH_SECRET`
5. [ ] Config: swap `JWT_SECRET`/`REFRESH_SECRET` → `CLERK_SECRET_KEY`/`CLERK_PUBLISHABLE_KEY` + app domain
6. [ ] Tests: rewrite auth snapshots (`can_login`, `can_get_current_user`, reset-password), add mock-JWKS Clerk harness

## Epic 4 — Test coverage (unit + integration per module) · owner @qa-tester

Current: ~40 test fns, uneven (users=10, auth=8, security=7; most others 1–2). 11 of 18 controllers and 11 of 17 model entities have **zero** tests. @qa-tester found 12 orphaned `mod.rs` declarations (6 models + 6 requests) that pointed at test files never committed — proof the stubs were planned but never written.

Common acceptance bar (every controller ticket): happy-path + auth-failure (missing/invalid JWT → guest-degrade or 401) + ownership/IDOR boundary (user B cannot read/mutate user A's resource; expect 404, not leak). Each ticket restores its own `mod` declaration as the file lands. Follow existing patterns in `tests/requests/user.rs` + `tests/models/users.rs` (snapshot where sensible, `prepare_data` helpers).

- [ ] **T4.1 — Cart + Order request tests (security-critical, pairs with Epic 1)** `tests/requests/carts.rs` + `tests/requests/order.rs`. Cover: cart CRUD bound to authenticated user (IDOR → 404); `order_add` sets `final_price` server-side from `product.price` (client-supplied price ignored); `order_list`/`get_order_one` ownership; `order_confirm`; seller-join fix visible via order endpoints.
- [ ] **T4.2 — Seller request tests** `tests/requests/seller.rs`. Seller order listing returns only own `order_items` (join fix: `oi.seller_id`); cannot see other sellers' orders/items.
- [ ] **T4.3 — Delivery domain request tests** `delivery_method.rs` + `delivery_method_service.rs` + `delivery_address.rs`. CRUD each; `delivery_address` is JWT-bound (ownership + IDOR); `delivery_method`/`_service` list guest-readable, mutations auth-checked.
- [ ] **T4.4 — Payment + meetup request tests** `payment_methods.rs` + `meetup_address.rs`. CRUD + auth boundaries.
- [ ] **T4.5 — Catalog request tests** `brands.rs` + `materials.rs`. CRUD; guest list, auth for mutations. Deepen `products.rs`/`categories.rs` happy-path where thin.
- [ ] **T4.6 — Upload tests** unit test for `generate_unique_filename` (collision handling) + multipart upload happy-path if harness allows.
- [ ] **T4.7 — Model-layer unit tests (11 zero-coverage entities)** `carts`, `orders`, `order_items`, `delivery_methods`, `delivery_method_services`, `delivery_addresses`, `payment_methods`, `meetup_addresses`, `brands`, `materials`, `refresh_sessions`. find/CRUD/relations per entity.
- [ ] **T4.8 — Green suite gate** `cargo test` compiles + passes end-to-end; zero orphaned `mod` declarations; confirm 12 removed decls are re-added only as their files exist.
- [ ] Target (done = all of T4.1–T4.8): every controller has ≥ happy-path + auth-failure + ownership test; every model entity has ≥ find/CRUD test.

## Epic 5 — Release process: semver + tag-from-main + CI

- [ ] Adopt semantic versioning (MAJOR.MINOR.PATCH). Current version in `Cargo.toml` = baseline `0.x`.
- [ ] Release branch = `main` (note: GitHub default is `main`, NOT `master` — confirm rename vs. use `main`).
- [ ] Tag `vX.Y.Z` on `main` = deploy trigger to VPS. `develop` → PR → squash-merge to `main` → tag.
- [ ] CI prerequisite (none exists today): on PR + on tag, run `cargo test`; block tag if red.
- [ ] Deploy automation: tag → build on VPS → `systemctl restart seco`.

## Epic 6 — Defects surfaced by @qa-tester test coverage (fix before merge)

All verified against source by @techlead (2026-09-08). Four are same-class as Epic 1 (auth/IDOR) and are must-fix-before-merge.

- [ ] **`delivery_address_update` (PUT/PATCH) — unauthenticated IDOR** (confirmed: no `auth::JWT`, `load_item(id)` only, no `find_by_id_and_user_id`). Anyone can overwrite any address. Fix: add JWT + ownership check mirroring `delivery_address_remove`. Owner @software-developer.
- [ ] **`DeliveryAddressResponse::new` copy-paste** (confirmed: `phone`/`email`/`city`/`postal_code`/`address` all copy `.name`). Fix: map real columns. Owner @software-developer.
- [ ] **`delivery_method_add` guest-open** (confirmed: no auth while update/remove already require superuser). Fix: add superuser guard. Owner @software-developer.
- [ ] **`brands` + `materials` add/update/remove guest-open** (confirmed). Fix: superuser guard on all mutations; `list`/`get_one` stay guest. Owner @software-developer.
- [ ] **`get_all_products` / `get_product_by_id` use `INNER JOIN categories`** while `category_id` is nullable → uncategorized products invisible in search (and inconsistent with `get_all_products_by_user_id`, which doesn't join categories). Fix: `LEFT JOIN categories` (matches nullable schema; `brands`/`materials` already LEFT JOIN). Owner @software-developer.
- [ ] **`meetup_address` controller dead** (declared in `controllers/mod.rs`, never mounted in `app.rs`; also fully guest-open + client-supplied `user_id`). DECISION DEFERRED to @user: mount-with-auth vs delete vs defer. Do NOT mount as-is.

## Users / note

No Firebase — `firebase_id` is a legacy string column only. No live chat/negotiation backend.

---

*Maintained by @techlead. Assignments via `@name` in the room; issues mirrored to `treeclouds/seco` once `gh` token lands.*
