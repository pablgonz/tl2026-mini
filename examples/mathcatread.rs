// Lee fórmulas MathML (una por línea) desde stdin y escribe su lectura con MathCAT.
// Preferencias por variables de entorno: LANG_MC (es), STYLE (ClearSpeak), VERBOSITY (Medium).
// mathcatread: lee MathML con la librería de MathCAT, sin NVDA ni navegador.
//
// Entrada: MathML por la entrada estándar, una fórmula por línea (las líneas vacías se saltan).
// Salida:  una línea por fórmula. Los errores y los panic de MathCAT salen como "ERROR: ...".
// Usa las reglas de la carpeta Rules/ de este repo (YAML se lee al ejecutar) y el código de src/
//   tal como está compilado, así que hay que recompilar después de tocar src/:
//     cargo build --example mathcatread
//
// Preferencias (variables de entorno, entre paréntesis el valor por defecto):
//   LANG_MC (es)  STYLE (ClearSpeak)  VERBOSITY (Medium)  TTS (None)
//   CAPS, CAPS_WORD  letras mayúsculas (SpeechOverrides_CapitalLetters, CapitalLetters_UseWord)
//   PREFS="Clave=valor;Clave=valor"   cualquier otra preferencia de MathCAT
//
// Modos (si no pones ninguno, da la lectura normal):
//   NAV="ZoomIn MoveNext ..."   recorre la fórmula con esos comandos y muestra cada paso
//                               (NAVMODE=Enhanced, NAVVERB=Medium, AUTOZOOM=True)
//   BRAILLE=UEB                 braille con ese código (UEB, Nemeth, CMU, ...), en Unicode y en puntos
//   OVERVIEW=1                  resumen de la fórmula
//   CANON=1                     MathML canonicalizado (lo que ve MathCAT después de set_mathml)
//
// Ejemplos:
//   echo '<math><mn>1</mn><mtext>º</mtext></math>' | cargo run -q --example mathcatread
//   echo '<math><mfrac><mi>a</mi><mi>b</mi></mfrac></math>' | LANG_MC=en NAV="ZoomIn MoveNext" cargo run -q --example mathcatread
// Más cómodo: ./mcread -h
use libmathcat::interface::*;
use std::io::BufRead;
fn main() {
    set_rules_dir(concat!(env!("CARGO_MANIFEST_DIR"), "/Rules")).unwrap();
    let env = |k: &str, d: &str| std::env::var(k).unwrap_or(d.to_string());
    set_preference("TTS", env("TTS", "None")).unwrap();
    set_preference("SpeechOverrides_CapitalLetters", env("CAPS", "")).unwrap();
    set_preference("CapitalLetters_UseWord", env("CAPS_WORD", "false")).unwrap();
    set_preference("Language", env("LANG_MC", "es")).unwrap();
    set_preference("SpeechStyle", env("STYLE", "ClearSpeak")).unwrap();
    set_preference("Verbosity", env("VERBOSITY", "Medium")).unwrap();
    for kv in env("PREFS", "").split(';').filter(|s| !s.is_empty()) { let (k, v) = kv.split_once('=').unwrap(); set_preference(k, v).unwrap(); }
    for line in std::io::stdin().lock().lines() {
        let line = line.unwrap();
        if line.trim().is_empty() { continue; }
        // NAV="ZoomIn MoveNext ...": recorre la fórmula con esos comandos e imprime cada paso
        let nav = env("NAV", "");
        if !nav.is_empty() {
            set_preference("NavMode", env("NAVMODE", "Enhanced")).unwrap();
            set_preference("NavVerbosity", env("NAVVERB", "Medium")).unwrap();
            set_preference("AutoZoomOut", env("AUTOZOOM", "True")).unwrap();
            if let Err(e) = set_mathml(line.clone()) { println!("ERROR: {e:#}"); continue; }
            let steps: Vec<String> = nav.split_whitespace().map(|c| {
                format!("{c}: {}", do_navigate_command(c).unwrap_or_else(|e| format!("ERROR: {e:#}")))
            }).collect();
            println!("{}", steps.join(" | "));
            continue;
        }
        let br = env("BRAILLE", "");
        if !br.is_empty() {
            set_preference("BrailleCode", br.clone()).unwrap();
            set_preference("BrailleNavHighlight", "Off").unwrap();
            let out = set_mathml(line).and_then(|_| get_braille(""));
            match out {
                Ok(b) => {
                    // también en puntos: 1-2-3 por celda, espacio = "0"
                    let dots: Vec<String> = b.chars().map(|c| {
                        let v = c as u32;
                        if (0x2800..=0x28FF).contains(&v) {
                            let m = v - 0x2800;
                            let d: String = (0..8).filter(|i| m & (1 << i) != 0).map(|i| char::from(b'1' + i as u8)).collect();
                            if d.is_empty() { "0".to_string() } else { d }
                        } else { c.to_string() }
                    }).collect();
                    println!("{b}\t{}", dots.join("-"));
                }
                Err(e) => println!("ERROR: {e:#}"),
            }
            continue;
        }
        if env("CANON","")=="1" { println!("{}", set_mathml(line.clone()).unwrap_or_default()); continue; }
        let out = set_mathml(line).and_then(|_| if env("OVERVIEW","")=="1" { get_overview_text() } else { get_spoken_text() });
        println!("{}", out.unwrap_or_else(|e| format!("ERROR: {e:#}")));
    }
}
