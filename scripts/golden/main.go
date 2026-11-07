// Golden-vector exporter for the owned Go reference. Rust compiler labels and
// values are the only deliberate renderer substitution. No application inputs.
package main

import (
    "bytes"
    "encoding/json"
    "flag"
    "os"
    "path/filepath"
    "time"
    banner "golden-reference/banner"
)
type RenderVector struct {
    Language string `json:"language"`
    Structured bool `json:"structured"`
    Started bool `json:"started"`
    Want string `json:"want"`
}
type RedactVector struct {
    Args []string `json:"args"`
    Extra []string `json:"extra"`
    Want []string `json:"want"`
}
type QuoteVector struct { Command string `json:"command"`; Args []string `json:"args"`; Want string `json:"want"` }
func main() {
    output := flag.String("output", "", "Output JSON snapshot")
    locales := flag.String("locales", "", "Go reference catalogs")
    flag.Parse()
    var vectors struct { Render []RenderVector `json:"render"`; Redact []RedactVector `json:"redact"`; Quote []QuoteVector `json:"quote"`; Completion string `json:"completion"` }
    for _, language := range []string{"en","es","pt-BR"} {
        data, err := os.ReadFile(filepath.Join(*locales,language+".json")); must(err)
        var text banner.Translations; must(json.Unmarshal(data,&text)); text.Build.GoVersion="Rust"
        for _, structured := range []bool{false,true} {
            for _, started := range []bool{false,true} {
                config := banner.Config{Name:"MyApp",Description:"Database collector",Version:"1.2.3+20261107.42",BuildTime:"2026-11-07T12:34:56-03:00",Author:"Application author",Command:"myapp",Args:[]string{"--password","hidden","two words","--file","inventory.lst"},Language:banner.Language(language),Translations:&text,NoColor:true}
                if started { config.StartedAt=time.Date(2026,11,7,13,14,15,123456789,time.FixedZone("caller",-3*60*60)) }
                if structured { config.Build=&banner.BuildDetails{Version:config.Version,BuildDate:config.BuildTime,OverallBuildNumber:792,Commit:"abcdef0123456789abcdef0123456789abcdef01",GoVersion:"rustc 1.99.1"}; config.Version="";config.BuildTime="" }
                var buf bytes.Buffer; must(banner.Write(&buf,config)); vectors.Render=append(vectors.Render,RenderVector{language,structured,started,buf.String()})
            }
        }
    }
    for _, tc := range []RedactVector{
        {Args:[]string{"--password","database-secret","--PASSWORD=inline-secret","--db-token=token-secret","--wallet","private-wallet"}},
        {Args:[]string{"-p=short-secret","-p","other-secret","--config-file=private.toml","--inventory-file","private.lst"},Extra:[]string{"-p","--config-file","--inventory-file"}},
        {Args:[]string{"--file","inventory.csv","--tns-admin",`C:\Oracle Network\Admin`,`--tns-admin=C:\Other Network\Admin`}},
        {Args:[]string{"--endpoint=oracle://scott:tiger@db.example/svc?token=abc","oracle://u:p@db/svc?name=ok&password=hidden"}},
        {Args:[]string{"https://db/path?api-key=hidden&safe=yes"},Extra:[]string{"--api-key"}},
        {Args:[]string{"--endpoint=oracle://user:secret@[bad","oracle://user:secret@db/svc?token=%ZZ","https://db/path?token=secret;bad=yes"}},
        {Args:[]string{"--pass\x00word","hidden","value\nnext\tend\x1b"}},
        {Args:[]string{"--password"}},
        {Args:[]string{"https://u%40ser:p%40ss@HOST:443/a%2Fb?b=2&a=1&a=3&token=x&token=y#part"}},
        {Args:[]string{"HTTPS://U:P@EXAMPLE.COM/path?q=space+value&private-key=x"}},
        {Args:[]string{"https://db/?%74oken=x&safe=%FF&blank&x=a%3Bb&x=%2B"}},
        {Args:[]string{"https://db/path?","https://db/path?&&","https://db/path#"}},
        {Args:[]string{"https://db/a b?query=a b&password=x"}},
        {Args:[]string{"--token","--safe-looking","-password=unchanged","-abc","--","--wallet-location=x"}},
        {Args:[]string{"https://u:p@[::1]:8443/x?TOKEN=x","oracle://u:p@db/service?b=2&a=1"}},
        {Args:[]string{"--endpoint=https://db/a%ZZ?token=secret","https://user:pass@host/#bad%ZZ"}},
        {Args:[]string{"--PRİVATE-KEY=hidden-inline","--PRİVATE-KEY","hidden-separate","https://host/?PR%C4%B0VATE-KEY=hidden-query"}},
        {Args:[]string{"--İD=hidden-id","--İD","hidden-id-separate","https://host/?%C4%B0D=hidden-id-query"},Extra:[]string{"--id"}},
        {Args:[]string{"--id=hidden-custom","--id","hidden-custom-separate","https://host/?id=hidden-custom-query"},Extra:[]string{"--İD"}},
        {Args:[]string{"--ΟΣ=hidden-sigma","--ΟΣ","hidden-sigma-separate","https://host/?%CE%9F%CE%A3=hidden-sigma-query","--ος=safe-final-sigma"},Extra:[]string{"--οσ"}},
    } { tc.Want=banner.RedactArgs(tc.Args,tc.Extra...); vectors.Redact=append(vectors.Redact,tc) }
    for _, tc := range []QuoteVector{
        {Command:"my app", Args:[]string{"","two words","a\"b","--password","hidden"}},
        {Command:"tool\n", Args:[]string{"value\r", "\u00a0", "\u2028", "\u3000", "C:\\two words\\file"}},
        {Command:"tool", Args:[]string{"a\\b", "quote\"and\\slash", "plain", "é", "😀"}},
        {Command:"tool", Args:[]string{"x\u200b y", "x\ufeff y", "x\ue000 y", "x\U000f0000 y", "x\u0378 y", "x\u2029 y"}},
    } { tc.Want=banner.CommandLine(tc.Command,tc.Args); vectors.Quote=append(vectors.Quote,tc) }
    var done bytes.Buffer;must(banner.WriteCompletion(&done));vectors.Completion=done.String()
    data,err:=json.MarshalIndent(vectors,"","  ");must(err);must(os.WriteFile(*output,append(data,'\n'),0644))
}
func must(err error) { if err != nil { panic(err) } }
