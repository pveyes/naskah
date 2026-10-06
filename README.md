# naskah [![Actions Status](https://github.com/pveyes/naskah/workflows/build/badge.svg)](https://github.com/pveyes/naskah/actions)

> Bahasa pemrograman dengan sintaks Bahasa Indonesia

Demo: https://naskah.vercel.app/

## Tipe data

- angka `123`, `1.5`, `0xff`, `0b101`
- teks `"halo"`
- boolean `benar` / `salah`
- kosong `kosong`
- daftar `[1, 2, 3]`
- objek `{ nama: "Budi", umur: 3 }`

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

Gunakan tanda kurung `( )` untuk mengubah urutan. Komentar diawali `//`.

## Sintaks

### Variabel

```
misal x = 4;
konstan pi = 3.14;
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

Nama lain untuk daftar: `panjang` (`length`), `tambah` (`push`), `gabung` (`join`), `balik` (`reverse`). Nama ini diterjemahkan di mana pun dipakai, termasuk sebagai nama properti objek.

### Fungsi

```
fungsi jumlah(a, b) {
  hasilkan a + b;
}

tulis(jumlah(1, 2));
```

`tulis` menjadi `console.log` dan `tanya` menjadi `prompt`.

### Kesalahan sintaks

Kesalahan dilaporkan dengan posisinya, misalnya `baris 1, kolom 11: ekspresi tidak lengkap, ditemukan `;``.

## Pengembangan

```sh
cargo test --workspace   # jalankan test
./scripts/dev.sh         # build demo dan jalankan di http://localhost:8787
```
