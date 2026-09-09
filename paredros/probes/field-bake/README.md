# GPU field bake receipt

This is R2 from Mesocosm's engine/ecology review. It proves the smallest live
path from a Burn-produced resident field to a bounded renderling draw:

```text
Burn field
  -> CubeCL count / prefix / scatter
  -> CubeCL vertex buffer
  -> one device-local copy
  -> GPU-only renderling vertex range
  -> bounded renderling draw
```

The copy is intentional. CubeCL 0.10 can reuse a host-created wgpu device, but
its compute server cannot import an arbitrary renderling slab buffer as a
CubeCL handle. The result uses one device and two allocators. Vertex contents
never stage through the CPU; only the two-word compacted count and overflow
receipt return at the bake boundary.

Run the headed receipt with:

```powershell
$env:FIELD_BAKE_CAPTURE = '1'
cargo run --release
```

`FIELD_BAKE_CAPTURE_PATH` can override the PNG destination. The receipt checks
the Burn/CubeCL client identity, a deliberate undersized extraction, compacted
count, contiguous renderling capacity, bounded draw refusal, slab-growth
invalidation, publication buffer identity, and the presented image.

## Receipt

The 2026-08-21 headed run on an NVIDIA GeForce RTX 4060 Laptop GPU produced
7,182 compacted vertices from a 48 by 48 Burn field. A deliberate capacity of
7,181 raised the GPU overflow flag. The retained renderling allocation held
13,824 full vertices in one contiguous range, survived a forced slab growth by
reattaching to the new buffer identity, and drew with zero CPU vertex staging.

The publication copied 746,928 device-local bytes. The two-word count and
overflow metadata crossed to the CPU at the bake boundary. The capture had 63
scene-region colours and 31.6% non-background coverage.

Full renderling `Vertex` used 2.6 times the bytes of a ten-word procedural ABI.
The result keeps the full ABI for bounded asynchronous bakes because it uses
renderling's existing draw path. A compact ABI stays gated on a high-density
consumer that justifies another shader contract.
