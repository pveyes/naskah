# naskah [![Actions Status](https://github.com/pveyes/naskah/workflows/build/badge.svg)](https://github.com/pveyes/naskah/actions)

> Bahasa pemrograman dengan sintaks Bahasa Indonesia

Naskah ditulis dalam Bahasa Indonesia dan diterjemahkan ke JavaScript. Coba langsung di browser: **https://naskah.fatihkalifa.workers.dev**

## Sekilas

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

| Naskah | Gunanya |
|---|---|
| `misal`, `konstan` | variabel |
| `jika` / `lain`, `pilih` / `kalau` | percabangan |
| `selama`, `ulang`, `untuk` | perulangan |
| `fungsi`, `hasilkan` | fungsi |
| `tunggu`, `tunda(ms)` | menunggu hasil, tanpa penanda `async` |
| `coba` / `tangkap` / `akhirnya`, `lempar` | menangani galat |
| `Hewan(nama) { }`, `.nama`, `..nama` | kelas, objek ini, dan versi induk |
| `"Halo, {nama}"` | teks dengan sisipan |

Huruf kapital hanya untuk kelas, dan memanggil nama berhuruf kapital selalu membuat objek baru. Titik koma boleh dihilangkan.

Penjelasan lengkap ada di **[SYNTAX.md](SYNTAX.md)**.

## Pengembangan

```sh
cargo test --workspace   # jalankan test
./scripts/dev.sh         # build demo dan jalankan di http://localhost:5173
```

## Penerapan

Demo diterapkan ke Cloudflare Workers dengan CLI [`cf`](https://www.npmjs.com/package/cf). Proyeknya ada di `demo/static`: Vite menyusun situsnya dan `cf deploy` mengunggahnya sebagai Worker bernama `naskah`, yang tersedia di `naskah.<subdomain-akun>.workers.dev` (saat ini `naskah.fatihkalifa.workers.dev`).

```sh
cf auth login            # sekali saja
./scripts/deploy.sh      # build lalu cf deploy
```

Push ke `master` menerapkannya otomatis lewat GitHub Actions. Ia membutuhkan dua secret di repositori: `CLOUDFLARE_API_TOKEN` dan `CLOUDFLARE_ACCOUNT_ID`.
