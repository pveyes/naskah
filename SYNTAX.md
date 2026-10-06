# Sintaks Naskah

Naskah ditulis dalam Bahasa Indonesia dan diterjemahkan ke JavaScript. Dokumen ini menjelaskan seluruh sintaksnya. Untuk gambaran singkat, lihat [README](README.md), dan untuk mencobanya langsung, buka playground di browser.

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
| `bukan` | bukan |
| `==` `!=` | sama / tidak sama |
| `>` `<` `>=` `<=` | perbandingan |
| `+` `-` | tambah, kurang |
| `*` `/` `%` | kali, bagi, sisa bagi |
| `-x` | negatif |
| `^` | pangkat |
| `a.b` `a[0]` `f(x)` | properti, elemen, panggil fungsi |
| `tunggu x` | tunggu hasil (setingkat dengan `-x`) |

Gunakan tanda kurung `( )` untuk mengubah urutan. Komentar diawali `//`.

## Sintaks

Sebuah pernyataan berakhir di titik koma, di akhir baris, atau sebelum `}`. Jadi `misal x = 1;` dan `misal x = 1` sama saja.

Karena baris baru mengakhiri pernyataan, tanda yang membuka baris (`.`, `(`, `[`, `-`, dan operator) tidak melanjutkan baris sebelumnya. Untuk memecah ekspresi panjang, akhiri baris dengan operatornya (`a +`, `xs.`), atau tulis di dalam `( )`, `[ ]`, dan `{ }`.

### Variabel

```
misal x = 4;
konstan pi = 3,14;
x = x + 1;
```

### Percabangan

```
jika x == 2 {
  tulis("dua");
} lain jika x > 2 dan bukan x == 10 {
  tulis("lebih dari dua");
} lain {
  tulis("kurang dari dua");
}
```

Untuk mengecek `kosong`, `benar` dan `salah` ada singkatan: `jika x kosong {` sama dengan `jika x == kosong {`.

### Perulangan

```
selama x < 10 {
  x = x + 1;
}

// tanpa kondisi, berhenti dengan `berhenti;`
ulang {
  jika x > 20 {
    berhenti;
  }
  x = x + 1;
  lanjut;
}
```

Perulangan dengan hitungan. Batas `sampai` ikut dihitung. Beri `langkah` negatif untuk menghitung mundur.

```
untuk i dari 1 sampai 10 {
  tulis(i);
}

untuk i dari 10 sampai 0 langkah -2 {
  tulis(i);
}
```

Perulangan untuk setiap isi daftar:

```
untuk setiap barang dalam belanja {
  tulis(barang);
}
```

### Pilihan

```
pilih x {
  kalau 1, 2 {
    tulis("satu atau dua");
  }
  kalau 3 {
    tulis("tiga");
  }
  lain {
    tulis("lainnya");
  }
}
```

Setiap `kalau` berdiri sendiri, tidak lanjut ke `kalau` di bawahnya. `berhenti;` dan `lanjut;` di dalam `kalau` berlaku untuk perulangan di sekitarnya.

### Daftar dan objek

```
misal belanja = ["beras", "telur"];
belanja.tambah("gula");
belanja[0] = "ketan";
tulis(belanja.panjang);

misal orang = { nama: "Budi", umur: 3 };
orang.umur = orang.umur + 1;
```

Nama khusus untuk daftar: `panjang` (`length`), `tambah` (`push`), `gabung` (`join`), `balik` (`reverse`). Nama ini diterjemahkan di mana pun dipakai, termasuk sebagai nama properti objek.

### Fungsi

```
fungsi jumlah(a, b) {
  hasilkan a + b;
}

tulis(jumlah(1, 2));
```

`tulis` menjadi `console.log` dan `tanya` menjadi `prompt`.

### Teks dengan sisipan

Apa pun di dalam `{ }` pada sebuah teks dihitung sebagai ekspresi.

```
misal nama = "Budi";
tulis("Halo, {nama}! Kamu punya {belanja.panjang + 1} barang.");
```

Tulis `\{` untuk kurung kurawal biasa: `"\{bukan sisipan}"`.

### Fungsi sebagai nilai

`fungsi` tanpa nama bisa dikirim sebagai argumen.

```
misal ganda = angka.peta(fungsi (x) {
  hasilkan x * 2;
});
```

Nama bawaan untuk daftar: `panjang` (`length`), `tambah` (`push`), `gabung` (`join`), `balik` (`reverse`), `peta` (`map`), `saring` (`filter`), `cari` (`find`), `urut` (`sort`).

### Tunggu

`tunggu` menunggu hasil yang datang nanti. Fungsi yang berisi `tunggu` otomatis menjadi `async`, tidak perlu penanda tambahan.

```
fungsi ambil() {
  tunggu tunda(300);   // tunda(ms) menunggu sekian milidetik
  hasilkan "data";
}

misal hasil = tunggu ambil();
```

### Menangani galat

```
coba {
  lempar Galat("jaringan putus");
} tangkap galat {
  tulis("Gagal: {galat.pesan}");
} akhirnya {
  tulis("selesai");
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

### Kesalahan sintaks

Kesalahan dilaporkan satu per satu dengan posisinya, misalnya `baris 1, kolom 11: ekspresi tidak lengkap, ditemukan `;``.

Nama yang dipakai tetapi belum pernah dibuat juga dilaporkan sebelum program berjalan, lengkap dengan tebakan kalau mirip salah ketik: `` `nmaa` belum dibuat. Maksudmu `nama`? ``. Nama hanya bisa dipakai di tempat ia dibuat, jadi nama yang dibuat di dalam `{ }` tidak terlihat di luarnya, dan parameter kelas (`Hewan(nama)`) hanya terlihat di konstruktor, bukan di metode.

Kesalahan yang baru ketahuan saat program berjalan, misalnya mengambil sesuatu dari nilai `kosong`, juga dijelaskan dalam Bahasa Indonesia beserta nomor barisnya.
