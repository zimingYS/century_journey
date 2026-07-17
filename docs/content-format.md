# Content Format

Century Journey content definitions use JSON files below assets/definitions.
Every definition file must declare the current top-level version:

    {
      "format_version": 1
    }

The current version is 1. Missing, older, and newer versions are rejected.
Changing this number requires an explicit loader or migration change.

## Validation

Run the content validator without launching the client:

    cargo run --locked -- --check-content assets

The command checks JSON parsing, format_version, duplicate identifiers,
textures, recipe inputs and outputs, biome block references, loot entries, and
tag members. It exits with a non-zero status when any error is found and runs as
an independent CI job.

## Override Order

The built-in asset directory has the lowest content priority. Additional
definition roots can be supplied through the platform path-list environment
variable CJ_CONTENT_OVERRIDES.

    $env:CJ_CONTENT_OVERRIDES = "packs\base;packs\local"

Sources are evaluated in this order:

1. CJ_ASSET_ROOT, or assets when it is unset.
2. Each CJ_CONTENT_OVERRIDES entry from left to right.
3. A later source replaces an earlier JSON file only when both use the exact
   same relative path below definitions.

Files with different paths accumulate. Defining the same identifier from two
different paths is an error, because that would make ownership ambiguous.

This contract currently covers JSON content definitions only. Texture, model,
script, binary compatibility, load hooks, and runtime code extension are not a
stable Mod API. The environment variable names and Rust traits may change
before that API is explicitly versioned.
