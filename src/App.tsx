import { useMemo, useState } from "react";
import { MonitorUp, Wifi, Settings, CheckCircle2 } from "lucide-react";

type Stage="setup"|"pairing"|"ready";

function makeCode(){const chars="ABCDEFGHJKLMNPQRSTUVWXYZ23456789";return Array.from({length:6},()=>chars[Math.floor(Math.random()*chars.length)]).join("");}

export default function App(){
 const [stage,setStage]=useState<Stage>("setup");
 const [serverUrl,setServerUrl]=useState("http://localhost:8092");
 const code=useMemo(makeCode,[]);
 const grouped=code.slice(0,3)+" "+code.slice(3);
 return <main className="shell">
   <header className="topbar"><div className="brand"><span className="mark"><span/></span><strong>DisplayHub</strong><em>Player</em></div><div className="status"><Wifi size={16}/> Player 0.1.0</div></header>
   <section className="content">
    {stage==="setup"&&<div className="card setup"><div className="heroIcon"><MonitorUp/></div><p className="eyebrow">Welcome to DisplayHub Player</p><h1>Turn this screen into a DisplayHub display.</h1><p className="lead">Connect this player to your DisplayHub server. Once paired, signage and screen sharing will start automatically.</p><label>DisplayHub server<input value={serverUrl} onChange={e=>setServerUrl(e.target.value)} placeholder="https://displayhub.example.com"/></label><button onClick={()=>setStage("pairing")} disabled={!serverUrl.trim()}>Connect to DisplayHub</button><small>You only need to do this once on each player.</small></div>}
    {stage==="pairing"&&<div className="card pairing"><p className="eyebrow">Pair this display</p><h1>Enter this code in DisplayHub</h1><div className="code">{grouped}</div><p className="lead">In your DisplayHub admin page, open <strong>Screens → Add Player</strong> and enter the code above.</p><div className="waiting"><span className="pulse"/><span>Waiting for pairing…</span></div><button className="secondary" onClick={()=>setStage("ready")}>Preview paired state</button><small>Prototype: server pairing will replace this preview button in the next step.</small></div>}
    {stage==="ready"&&<div className="card ready"><CheckCircle2 className="success"/><p className="eyebrow">Display connected</p><h1>DT Classroom</h1><div className="readyGrid"><div><span>DisplayHub</span><strong>Connected</strong></div><div><span>AirPlay</span><strong>Coming next</strong></div></div><p className="lead">This player is ready to load its assigned DisplayHub content.</p><button onClick={()=>setStage("pairing")}>Back to pairing</button></div>}
   </section>
   <footer><span>DisplayHub Player</span><span><Settings size={14}/> Device setup</span></footer>
 </main>
}