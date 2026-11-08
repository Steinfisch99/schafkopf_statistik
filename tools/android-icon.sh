#!/usr/bin/env bash
# Erzeugt das Android-App-Icon aus einer SVG-Quelle (siehe tools/README.md).
#
#   tools/android-icon.sh <quelle.svg> <ziel-ordner>
#
# Die Quelle hat eine viewBox von 0 0 108 108 (Raster der adaptiven Android-Icons) und ein Element pro Zeile:
# `<rect>`, `<circle>` oder `<path>` mit fill/stroke-Attributen, ohne transform, ohne Gruppen. Das Element mit
# id="bg" ist der Hintergrund, alles andere der Vordergrund. Sichtbar ist sicher nur der Kreis mit Radius 33
# um die Mitte (54,54); was darüber hinausgeht, schneidet manches Handy ab.
#
# Im Ziel-Ordner entstehen dieselben Dateinamen, die dx (Dioxus 0.7) mit seinem Platzhalter-Icon ins
# Gradle-Projekt schreibt. build-apk.sh kopiert sie nach `dx build` darüber und lässt Gradle noch einmal laufen:
#
#   drawable/ic_launcher_background.xml   Vektor (Android 8+)
#   drawable-v24/ic_launcher_foreground.xml
#   mipmap-*/ic_launcher.webp             Bild für Android 7 (ganze Fläche, abgerundet)
#
# Braucht resvg und cwebp, in der VM etwa:
#   nix-shell -I nixpkgs=<...> -p resvg libwebp --run "tools/android-icon.sh assets/icon.svg crates/x/android/icon"

set -euo pipefail

src="$1"
out="$2"
tmp="$(mktemp -d)"
trap 'rm -rf "$tmp"' EXIT

# SVG-Elemente zu VectorDrawable-Pfaden. Aufruf mit want=bg oder want=fg.
to_vector() {
  awk -v want="$1" '
    function attr(name,   m) {
      if (match(line, " " name "=\"[^\"]*\"")) {
        m = substr(line, RSTART, RLENGTH)
        sub("^ " name "=\"", "", m)
        sub("\"$", "", m)
        return m
      }
      return ""
    }
    function rect_path(x, y, w, h, r) {
      if (r == "" || r + 0 == 0)
        return sprintf("M%g,%gH%gV%gH%gZ", x, y, x + w, y + h, x)
      return sprintf("M%g,%gH%gA%g,%g 0 0 1 %g,%gV%gA%g,%g 0 0 1 %g,%gH%gA%g,%g 0 0 1 %g,%gV%gA%g,%g 0 0 1 %g,%gZ",
        x + r, y, x + w - r, r, r, x + w, y + r, y + h - r, r, r, x + w - r, y + h,
        x + r, r, r, x, y + h - r, y + r, r, r, x + r, y)
    }
    BEGIN {
      print "<?xml version=\"1.0\" encoding=\"utf-8\"?>"
      print "<!-- Erzeugt von tools/android-icon.sh, nicht von Hand ändern. -->"
      print "<vector xmlns:android=\"http://schemas.android.com/apk/res/android\""
      print "    android:width=\"108dp\" android:height=\"108dp\""
      print "    android:viewportWidth=\"108\" android:viewportHeight=\"108\">"
    }
    /<(rect|circle|path) / {
      line = $0
      is_bg = (attr("id") == "bg")
      if ((want == "bg") != is_bg) next
      if (line ~ /<rect /)
        d = rect_path(attr("x") + 0, attr("y") + 0, attr("width") + 0, attr("height") + 0, attr("rx"))
      else if (line ~ /<circle /) {
        cx = attr("cx") + 0; cy = attr("cy") + 0; r = attr("r") + 0
        d = sprintf("M%g,%gA%g,%g 0 1 0 %g,%gA%g,%g 0 1 0 %g,%gZ", cx - r, cy, r, r, cx + r, cy, r, r, cx - r, cy)
      } else
        d = attr("d")
      if (line ~ / transform=/) { print "transform wird nicht unterstützt: " line > "/dev/stderr"; exit 1 }
      s = "    <path android:pathData=\"" d "\""
      fill = attr("fill")
      if (fill == "") fill = "#000000"
      if (fill != "none") s = s " android:fillColor=\"" fill "\""
      stroke = attr("stroke")
      if (stroke != "" && stroke != "none") {
        sw = attr("stroke-width"); if (sw == "") sw = 1
        s = s " android:strokeColor=\"" stroke "\" android:strokeWidth=\"" sw "\""
        if (attr("stroke-linecap") != "") s = s " android:strokeLineCap=\"" attr("stroke-linecap") "\""
        if (attr("stroke-linejoin") != "") s = s " android:strokeLineJoin=\"" attr("stroke-linejoin") "\""
      }
      print s " />"
      n++
    }
    END {
      print "</vector>"
      if (n == 0) { print "kein Element für " want > "/dev/stderr"; exit 1 }
    }
  ' "$src"
}

mkdir -p "$out/drawable" "$out/drawable-v24"
to_vector bg > "$out/drawable/ic_launcher_background.xml"
to_vector fg > "$out/drawable-v24/ic_launcher_foreground.xml"

# Altes Bild-Icon: der sichtbare Ausschnitt (wie bei den adaptiven Icons) mit runden Ecken.
sed -e 's|viewBox="0 0 108 108"|viewBox="18 18 72 72"|' \
  -e 's|<svg \([^>]*\)>|<svg \1><clipPath id="clip"><rect x="18" y="18" width="72" height="72" rx="14"/></clipPath><g clip-path="url(#clip)">|' \
  -e 's|</svg>|</g></svg>|' "$src" > "$tmp/legacy.svg"
for pair in mdpi:48 hdpi:72 xhdpi:96 xxhdpi:144 xxxhdpi:192; do
  dpi="${pair%%:*}"
  px="${pair##*:}"
  mkdir -p "$out/mipmap-$dpi"
  resvg -w "$px" -h "$px" "$tmp/legacy.svg" "$tmp/$dpi.png"
  cwebp -quiet -lossless "$tmp/$dpi.png" -o "$out/mipmap-$dpi/ic_launcher.webp"
done
echo "Icon nach $out geschrieben"
