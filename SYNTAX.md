# Sintaks Naskah

Naskah ditulis dalam Bahasa Indonesia dan diterjemahkan ke JavaScript. Dokumen ini menjelaskan seluruh sintaksnya. Untuk gambaran singkat, lihat [README](README.md). Untuk belajar dari awal, ada pelajaran bergambar contoh di halaman Belajar, dan untuk mencobanya langsung, buka Tempat Coba di peramban.

- [Tipe data](#tipe-data)
- [Operator](#operator)
- [Sintaks](#sintaks): [variabel](#variabel), [percabangan](#percabangan), [perulangan](#perulangan), [pilihan](#pilihan), [daftar dan objek](#daftar-dan-objek), [fungsi](#fungsi), [teks dengan sisipan](#teks-dengan-sisipan), [fungsi sebagai nilai](#fungsi-sebagai-nilai), [tunggu](#tunggu), [menangani galat](#menangani-galat), [kelas](#kelas), [blok sebagai nilai](#blok-sebagai-nilai), [huruf kapital](#huruf-kapital), [kesalahan sintaks](#kesalahan-sintaks)

## Tipe data

- angka `123`, `1,5`, `0xff`, `0b101`
- teks `"halo"`, dengan sisipan `"Halo, {nama}!"`
- boolean `benar` / `salah`
- kosong `kosong`
- daftar `[1, 2, 3]`
- objek `{ nama: "Budi", umur: 3 }`

Angka desimal ditulis dengan koma seperti di sekolah: `3,14`. Koma di antara dua angka tanpa spasi adalah koma desimal, jadi beri spasi setelah koma pemisah: `[1, 2, 3]` dan `jumlah(1, 5)`. Menulis `[1,2,3]` dilaporkan sebagai galat karena bisa dibaca dua cara, begitu juga `1.5` dengan titik, lengkap dengan cara menulisnya yang benar. Hasil di Keluaran juga memakai koma.

## Operator

Dari yang paling lemah ke paling kuat:

| Operator | Arti |
|---|---|
| `=` | isi ulang variabel |
| `atau` | atau |
| `dan` | dan |
| `adalah` `bukan` | sama / tidak sama |
| `>` `<` `>=` `<=` | perbandingan |
| `+` `-` | tambah, kurang |
| `*` `/` `%` | kali, bagi, sisa bagi |
| `×` `÷` | kali, bagi (tanda dari buku sekolah, sama dengan `*` dan `/`) |
| `-x` | negatif |
| `^` | pangkat |
| `a.b` `a[0]` `f(x)` | properti, elemen, panggil fungsi |
| `tunggu x` | tunggu hasil (setingkat dengan `-x`) |

`adalah` dan `bukan` menggantikan `==` dan `!=`, dan `=` hanya untuk mengisi variabel. `bukan` tidak dipakai di depan sebuah nilai; untuk memeriksa salah, tulis `x adalah salah`.

Gunakan tanda kurung `( )` untuk mengubah urutan. Komentar diawali `//`.

## Sintaks

Sebuah pernyataan berakhir di titik koma, di akhir baris, atau sebelum `}`. Jadi `misal x = 1;` dan `misal x = 1` sama saja.

Karena baris baru mengakhiri pernyataan, tanda yang membuka baris (`.`, `(`, `[`, `-`, dan operator) tidak melanjutkan baris sebelumnya. Untuk memecah ekspresi panjang, akhiri baris dengan operatornya (`a +`, `xs.`), atau tulis di dalam `( )`, `[ ]`, dan `{ }`.

### Variabel

```
misal x = 4
konstan pi = 3,14
x = x + 1
```

### Percabangan

```
jika x adalah 2 {
  tulis("dua")
} lain jika x > 2 dan x bukan 10 {
  tulis("lebih dari dua")
} lain {
  tulis("kurang dari dua")
}
```

Untuk mengecek `kosong`, `benar` dan `salah` ada singkatan: `jika x kosong {` sama dengan `jika x adalah kosong {`.

### Perulangan

```
selama x < 10 {
  x = x + 1
}

// tanpa kondisi, berhenti dengan `berhenti`
ulang {
  jika x > 20 {
    berhenti
  }
  x = x + 1
  lanjut
}
```

Perulangan yang mengerjakan isinya dulu, lalu berhenti kalau syaratnya sudah benar. `sampai` harus satu baris dengan `}`.

```
misal x = 0
ulang {
  x = x + 1
} sampai x >= 5
```

Perulangan dengan hitungan. Batas `sampai` ikut dihitung, dan hitungannya selalu naik satu per satu. Untuk hitungan lain (mundur, loncat dua-dua), pakai `selama`.

```
untuk i dari 1 sampai 10 {
  tulis(i)
}

misal j = 10
selama j >= 0 {
  tulis(j)
  j = j - 2
}
```

Perulangan untuk setiap isi daftar:

```
untuk setiap barang dalam belanja {
  tulis(barang)
}
```

### Pilihan

```
pilih x {
  saat 1 / 2 {
    tulis("satu atau dua")
  }
  saat 3 {
    tulis("tiga")
  }
  lain {
    tulis("lainnya")
  }
}
```

Beberapa nilai dalam satu `saat` dipisah dengan `/`, dibaca "atau". Untuk pembagian di sana, pakai kurung (`saat (a / 2) {`) atau tulis `÷`.

Kalau isi sebuah `saat` (atau `lain`) hanya satu pernyataan, kurung kurawal boleh dilewatkan, asal pernyataan itu ditulis di baris yang sama:

```
pilih nilai {
  saat 100 tulis("Sempurna!")
  saat 80 / 90 tulis("Bagus sekali.")
  lain tulis("Terus berlatih.")
}
```

Setiap `saat` berdiri sendiri, tidak lanjut ke `saat` di bawahnya. `berhenti` dan `lanjut` di dalam `saat` berlaku untuk perulangan di sekitarnya.

### Daftar dan objek

```
misal belanja = ["beras", "telur"]
belanja.tambah("gula")
belanja[0] = "ketan"
tulis(belanja.panjang)

misal orang = { nama: "Budi", umur: 3 }
orang.umur = orang.umur + 1
```

Nama khusus untuk daftar: `panjang` (`length`), `tambah` (`push`), `gabung` (`join`), `balik` (`reverse`). Nama ini diterjemahkan di mana pun dipakai, termasuk sebagai nama properti objek.

### Fungsi

```
fungsi jumlah(a, b) {
  hasilkan a + b
}

tulis(jumlah(1, 2))
```

`tulis` menjadi `console.log` dan `tanya` menjadi `prompt`.

### Teks dengan sisipan

Apa pun di dalam `{ }` pada sebuah teks dihitung sebagai ekspresi.

```
misal nama = "Budi"
tulis("Halo, {nama}! Kamu punya {belanja.panjang + 1} barang.")
```

Tulis `\{` untuk kurung kurawal biasa: `"\{bukan sisipan}"`.

### Fungsi sebagai nilai

`fungsi` tanpa nama bisa dikirim sebagai argumen.

```
misal ganda = angka.ubah(fungsi (x) {
  hasilkan x * 2
})
```

Nama bawaan untuk daftar: `panjang` (`length`), `tambah` (`push`), `gabung` (`join`), `balik` (`reverse`), `ubah` (`map`), `saring` (`filter`), `cari` (`find`), `urut` (`sort`).

### Tunggu

`tunggu` menunggu hasil yang datang nanti. Fungsi yang berisi `tunggu` otomatis menjadi `async`, tidak perlu penanda tambahan.

```
fungsi ambil() {
  tunggu tunda(300)   // tunda(ms) menunggu sekian milidetik
  hasilkan "data"
}

misal hasil = tunggu ambil()
```

### Menangani galat

```
coba {
  lempar Galat("jaringan putus")
} tangkap galat {
  tulis("Gagal: {galat.pesan}")
} akhirnya {
  tulis("selesai")
}
```

`tangkap` boleh tanpa nama, dan `coba` butuh `tangkap` atau `akhirnya`. `Galat` adalah `Error`, dan `pesan` adalah `message`.

### Kelas

Kelas ditulis seperti memanggil fungsi, dengan nama berhuruf kapital. Isi badan kelas adalah konstruktornya, dan `suara() { }` adalah metode.

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

misal kucing = Kucing("Tom")
tulis(kucing.suara())
```

- `.nama` adalah `nama` milik objek yang sedang dijalankan, dan `..nama` adalah versi milik kelas induk. Keduanya hanya bisa dipakai di dalam kelas, termasuk di dalam fungsi tanpa nama (`fungsi (x) { }`) dan teks bersisipan, tetapi tidak di dalam `fungsi nama() { }`.
- `turunan Hewan(nama)` menyebut kelas induk beserta argumen yang dikirim ke konstruktornya. Argumen itu dihitung sebelum objek ada, jadi tidak bisa memakai `.nama` atau `tunggu`.
- Metode dengan nama yang sama menggantikan metode induk, tanpa kata khusus. Pakai `..suara()` untuk tetap memanggil versi induk.
- Konstruktor dibuat jika badan kelas berisi pernyataan atau kelas punya induk.
- `tunggu` tidak bisa dipakai langsung di badan kelas, hanya di dalam metode.

### Blok sebagai nilai

Blok `{ ... }` yang dipakai sebagai nilai menjalankan isinya dan menghasilkan nilai dari `hasilkan`.

```
Hewan(nama) {
  .nama = { misal x = 5; hasilkan nama + x }
}
```

`{ nama: 1 }` dan `{}` adalah objek, selain itu adalah blok. `hasilkan` di dalam blok keluar dari blok itu saja, bukan dari fungsi di sekitarnya. `berhenti` dan `lanjut` tidak bisa menembus blok.

### Huruf kapital

Huruf kapital hanya untuk kelas. Nama kelas harus diawali huruf kapital, sedangkan variabel, fungsi, parameter, dan metode harus diawali huruf kecil.

Memanggil nama berhuruf kapital selalu membuat objek baru, tanpa kata `baru`: `Kucing("Tom")`, `Galat("rusak")`, `Date()`. Memanggil metode berhuruf kecil seperti `Math.max(1, 2)` tetap pemanggilan biasa.

Nama kelas ditulis seperti memanggil fungsi, jadi `hewan(nama) {` dengan huruf kecil dianggap salah tulis dan dilaporkan sebagai galat.

### Bertanya kepada pengguna

`tanya` menampilkan sebuah pertanyaan, menunggu pengguna mengetik jawaban, lalu memberikannya sebagai tulisan.

```
misal nama = tanya("Siapa namamu?")
tulis("Halo, {nama}!")
```

Jawaban berupa tulisan, kecuali kamu menyebut jenisnya di nilai kedua: `Tipe.Teks` (bawaan) atau `Tipe.Angka`. Dengan `Tipe.Angka`, jawaban diubah menjadi angka. Angka dalam format Indonesia dimengerti (`12`, `1,5`, `1.000`), dan kalau jawabannya bukan angka, `tanya` memberi `kosong` dan menampilkan pesan. `bilangan` masih bisa dipakai untuk mengubah tulisan yang sudah ada menjadi angka.

```
misal umur = tanya("Umurmu berapa?", Tipe.Angka)
tulis("Tahun depan kamu {umur + 1} tahun.")
```

Menunggu jawaban tidak dihitung dalam batas waktu program. Kalau pengguna menghentikan program saat ia bertanya, `tanya` memberi `kosong`. `tanya` hanya bekerja di halaman yang memenuhi syarat keamanan peramban (lihat `public/_headers`); di tempat lain ia memberi `kosong` dan menampilkan pesan.

### Kesalahan sintaks

Kesalahan dilaporkan satu per satu dengan posisinya, misalnya `baris 1, kolom 11: ekspresi tidak lengkap, ditemukan `;``.

Nama yang dipakai tetapi belum pernah dibuat juga dilaporkan sebelum program berjalan, lengkap dengan tebakan kalau mirip salah ketik: `` `nmaa` belum dibuat. Maksudmu `nama`? ``. Nama hanya bisa dipakai di tempat ia dibuat, jadi nama yang dibuat di dalam `{ }` tidak terlihat di luarnya, dan parameter kelas (`Hewan(nama)`) hanya terlihat di konstruktor, bukan di metode.

Kesalahan yang baru ketahuan saat program berjalan, misalnya mengambil sesuatu dari nilai `kosong`, juga dijelaskan dalam Bahasa Indonesia beserta nomor barisnya.
