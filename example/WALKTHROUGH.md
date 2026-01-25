# Example: Whittling Down Cloud Resources

This example demonstrates how to use `pluck-list` to triage a long list of cloud resources. Instead of just "filtering", we will "whittle down" the list to find the items that require human judgment.

## Step 1: Remove the "Noise"
You identify that anything containing `dev`, `test`, or `temp` is safe to ignore or terminate.

1. Run: `./target/release/pluck-list example/cloud_resources.txt`
2. Press `Enter` -> `String match`.
3. Type: `(dev|test|temp)`
4. Review the highlighted lines in the left buffer.
5. Press `Enter` to pluck.
6. Press `Tab` to switch to the **New_List** (right buffer).
7. Press `S` to Save As `transient_resources.txt`.

## Step 2: Extract the "Critical"
Now you want to isolate the production environment.

1. Press `Tab` to switch back to the **Prompt Buffer**.
2. Select `String match`.
3. Type: `prod-`
4. Press `Enter` to pluck.
5. Press `Tab` to switch to the **New_List**.
6. Press `S` to Save As `production_inventory.txt`.

## Step 3: Triage the "Ambiguous"
Now, look at the **Modified List** (left buffer). All the `prod-`, `dev-`, and `test-` items are gone. You are left with a much shorter list of "mental model" edge cases:

- `monolith-legacy-v4` (2019!)
- `project-omega-alpha`
- `data-migration-tool`
- `obsolete-auth-v1`
- `data-science-jupyter` (Expensive p3 instance!)

These are the items that would be hard to capture in a single `aws` or `grep` command. You can now visually review just these items and decide their fate.

## Conclusion
By "plucking" out the categories you already understand, you reduce the cognitive load required to find the items that actually matter.
