# naskah [![Actions Status](https://github.com/pveyes/naskah/workflows/build/badge.svg)](https://github.com/pveyes/naskah/actions)

> A programming language written in Indonesian

Naskah is written in Indonesian and translated to JavaScript. Try it in the browser: **https://naskah.dev**

Never written a program before? Start with **[Belajar Naskah](https://naskah.dev/belajar)**: twelve short lessons in Indonesian, with examples you can run on the spot.

## At a glance

```
Hewan(nama) {
  .nama = nama

  suara() {
    hasilkan "..."
  }
}

Kucing(nama) turunan Hewan(nama) {
  suara() {
    hasilkan "meong, bukan {..suara()}"
  }
}

misal daftar = [Hewan("Burung"), Kucing("Tom")]
untuk setiap h dalam daftar {
  tulis("{h.nama} bilang {h.suara()}")
}
```

| Naskah | Purpose |
|---|---|
| `misal`, `konstan` | variables |
| `jika` / `lain`, `pilih` / `saat` | branching |
| `selama`, `ulang`, `untuk` | loops |
| `fungsi`, `hasilkan` | functions |
| `tunggu`, `tunda(ms)` | wait for a result, no `async` marker |
| `coba` / `tangkap` / `akhirnya`, `lempar` | error handling |
| `Hewan(nama) { }`, `.nama`, `..nama` | class, the current object, and the parent's version |
| `"Halo, {nama}"` | text with interpolation |

Capital letters are for classes only, and calling a capitalised name always builds a new object. Semicolons are optional.

The full reference is in **[SYNTAX.md](SYNTAX.md)** (in Indonesian).

## Development

```sh
cargo test --workspace   # run the tests
./scripts/dev.sh         # build the demo and serve it at http://localhost:5173
```

## Deployment

The demo is deployed to Cloudflare Workers with the [`cf`](https://www.npmjs.com/package/cf) CLI. The project lives in `demo/static`: Vite builds the site and `cf deploy` uploads it as a Worker named `naskah`, served on the custom domain `naskah.dev`.

```sh
cf auth login            # once only
./scripts/deploy.sh      # build, then cf deploy
```

A push to `master` deploys it automatically through GitHub Actions, which needs two repository secrets: `CLOUDFLARE_API_TOKEN` and `CLOUDFLARE_ACCOUNT_ID`.
