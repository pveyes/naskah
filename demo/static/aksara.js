// Decoration for the home page: old Javanese letters (aksara) float behind the page like ink
// rising from a manuscript. They flee and swirl around the pointer, and a click bursts new ones
// out. Purely visual, so it is skipped when the reader asks for less motion.
const HURUF = [..."ꦲꦤꦕꦫꦏꦢꦠꦱꦮꦭꦥꦝꦗꦪꦚꦩꦒꦧꦔ"];
const FONT = '"Noto Sans Javanese", serif';
const JARI = 170; // pointer influence radius, px

const kanvas = document.getElementById("aksara");
const diam = window.matchMedia("(prefers-reduced-motion: reduce)").matches;

if (kanvas && !diam) {
  const ctx = kanvas.getContext("2d");
  const warna = { dasar: "#8c2f1b", terang: "#c0472a" };
  const huruf = () => HURUF[Math.floor(Math.random() * HURUF.length)];
  let lebar = 0;
  let tinggi = 0;
  let butir = [];
  const jejak = [];
  const mouse = { x: -999, y: -999, kena: false };

  const bacaWarna = () => {
    const css = getComputedStyle(document.documentElement);
    const gelap = window.matchMedia("(prefers-color-scheme: dark)").matches;
    warna.dasar = css.getPropertyValue("--tok-keyword").trim() || warna.dasar;
    warna.terang = gelap ? "#ffb08a" : "#d4421f";
  };

  const buatButir = (awal) => ({
    c: huruf(),
    x: Math.random() * lebar,
    y: awal ? Math.random() * tinggi : tinggi + 60,
    vx: 0,
    vy: 0,
    naik: 0.15 + Math.random() * 0.35,
    ukuran: 28 + Math.random() * 70,
    alfa: 0.1 + Math.random() * 0.14,
    putar: (Math.random() - 0.5) * 0.4,
    sudut: (Math.random() - 0.5) * 0.6,
    pijar: 0,
  });

  const ukur = () => {
    const dpr = Math.min(window.devicePixelRatio || 1, 2);
    lebar = window.innerWidth;
    // Only as tall as the heading plus the top half of the code box; the CSS mask fades the rest.
    const wrap = document.querySelector(".playground-wrap");
    const kotak = wrap ? wrap.getBoundingClientRect() : null;
    tinggi = kotak
      ? Math.round(kotak.top + window.scrollY + kotak.height * 0.5)
      : window.innerHeight;
    kanvas.style.height = `${tinggi}px`;
    kanvas.width = lebar * dpr;
    kanvas.height = tinggi * dpr;
    ctx.setTransform(dpr, 0, 0, dpr, 0, 0);
    const jumlah = Math.round(Math.min(70, Math.max(24, lebar / 24)) * Math.min(1, tinggi / 800));
    while (butir.length < jumlah) butir.push(buatButir(true));
    butir.length = jumlah;
  };

  const semburan = (x, y, n) => {
    for (let i = 0; i < n; i++) {
      const arah = (i / n) * Math.PI * 2 + Math.random() * 0.4;
      const laju = 3 + Math.random() * 6;
      jejak.push({
        c: huruf(),
        x,
        y,
        vx: Math.cos(arah) * laju,
        vy: Math.sin(arah) * laju,
        ukuran: 26 + Math.random() * 40,
        umur: 1,
        sudut: Math.random() * 6,
      });
    }
  };

  window.addEventListener("pointermove", (e) => {
    mouse.x = e.pageX;
    mouse.y = e.pageY;
    mouse.kena = true;
  });
  document.addEventListener("pointerleave", () => (mouse.kena = false));
  window.addEventListener("pointerdown", (e) => semburan(e.pageX, e.pageY, 16));
  window.addEventListener("resize", ukur);
  const wrap = document.querySelector(".playground-wrap");
  if (wrap) new ResizeObserver(ukur).observe(wrap);
  window
    .matchMedia("(prefers-color-scheme: dark)")
    .addEventListener("change", bacaWarna);

  const gambar = (c, x, y, ukuran, sudut, alfa, isi) => {
    ctx.save();
    ctx.translate(x, y);
    ctx.rotate(sudut);
    ctx.globalAlpha = alfa;
    ctx.fillStyle = isi;
    ctx.font = `${ukuran}px ${FONT}`;
    ctx.fillText(c, 0, 0);
    ctx.restore();
  };

  const putar = () => {
    ctx.clearRect(0, 0, lebar, tinggi);
    ctx.textAlign = "center";
    ctx.textBaseline = "middle";

    for (let i = 0; i < butir.length; i++) {
      const b = butir[i];
      if (mouse.kena) {
        const dx = b.x - mouse.x;
        const dy = b.y - mouse.y;
        const jarak = Math.hypot(dx, dy);
        if (jarak < JARI && jarak > 0.1) {
          const k = 1 - jarak / JARI;
          // Push away, plus a sideways swirl so they curl around the pointer.
          b.vx += ((dx / jarak) * 2.4 - (dy / jarak) * 1.2) * k;
          b.vy += ((dy / jarak) * 2.4 + (dx / jarak) * 1.2) * k;
          b.pijar = Math.max(b.pijar, k);
        }
      }
      b.vx *= 0.94;
      b.vy += (-b.naik - b.vy) * 0.03;
      b.x += b.vx;
      b.y += b.vy;
      b.sudut += b.putar * 0.01 + b.vx * 0.01;
      b.pijar *= 0.95;
      if (b.y < -80 || b.x < -80 || b.x > lebar + 80) butir[i] = buatButir(false);
      const hampir = b.pijar > 0.05;
      gambar(
        b.c,
        b.x,
        b.y,
        b.ukuran * (1 + b.pijar * 0.35),
        b.sudut,
        Math.min(0.95, b.alfa + b.pijar * 0.7),
        hampir ? warna.terang : warna.dasar,
      );
    }

    for (let i = jejak.length - 1; i >= 0; i--) {
      const j = jejak[i];
      j.x += j.vx;
      j.y += j.vy;
      j.vx *= 0.96;
      j.vy *= 0.96;
      j.umur -= 0.016;
      if (j.umur <= 0) {
        jejak.splice(i, 1);
        continue;
      }
      gambar(j.c, j.x, j.y, j.ukuran, j.sudut, j.umur * 0.8, warna.terang);
    }

    requestAnimationFrame(putar);
  };

  bacaWarna();
  ukur();
  const mulai = () => requestAnimationFrame(putar);
  // Wait for the font, or the first frames draw empty boxes.
  document.fonts
    .load(`40px "Noto Sans Javanese"`, HURUF.join(""))
    .then(mulai, mulai);
}
