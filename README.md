# naskah

> Bahasa pemrograman dengan sintaks Bahasa Indonesia

Demo: https://naskah.netlify.com/

## Tipe data

Saat ini hanya 4 tipe data yang didukung oleh naskah:
 - angka `123`
 - huruf `"hello"`
 - boolean `benar` / `salah`
 - kosong `kosong`

## Operator

Operasi yang didukung oleh `naskah` adalah:
 - Penjumlahan `+`
 - Pengurangan `-`
 - Perkalian `*`
 - Pembagian `\`
 - Sisa pembagian `%`
 - Pangkat `^`

Selain itu ada juga operasi untuk membandingkan dua variabel / tipe data
 - Sama dengan `==`
 - Tidak sama dengan `!=`
 - Lebih dari `>`
 - Kurang dari `<`

## Sintaks

### Deklarasi variabel

```
misal x = 4;
misal y = x;
```

### Percabangan

```
jika x == 2 {
  
}

jika x == kosong {

}
```

Untuk kasus-kasus umum, naskah menyediakan sintaks khusus untuk pengecekan terhadap `kosong`, `benar` dan `salah`. Tidak perlu menulis operator `==`, cukup `x kosong`.

```
jika x kosong {

}
```

### Perulangan

Naskah saat ini hanya mempunyai 1 tipe perulangan yang tidak pernah berhenti

```
ulang {

}
```

Untuk berhenti di dalam perulangan, dapat menggunakan sintaks `berhenti;`

```
ulang {
  jika x > 2 {
    berhenti;
  }
}
```

## License 

MIT