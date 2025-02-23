import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from '@tauri-apps/api/event';
import "./App.css";

type ActiveWindow = {
  title: string;
  appName: string;
};

function App() {
  const [activeWindow, setActiveWindow] = useState<ActiveWindow>({
    title: "",
    appName: "",
  });
  const [message, setMessage] = useState("");
  const [appConfig, setAppConfig] = useState("");
  useEffect(() => {
    listen<ActiveWindow>('active-window-changed', (event) => {
      console.log(event);
      setActiveWindow(event.payload);
    });
  }, [])

  useEffect(() => {
    listen('serial-message', (event) => {
      console.log(event);
      const message = event.payload as string;
      console.log(`message: ${message}`);
      setMessage(message);
    });
  }, [])

  useEffect(() => {
    invoke('get_config').then((config) => {
      setAppConfig(config as string);
    });
  }, [])

  const saveConfig = async () => {
    await invoke('save_config', { configJson: appConfig });
  };

  return (
    <main className="container">
      <div>
        <h2>{activeWindow.appName}</h2>
        <p>{activeWindow.title}</p>
      </div>
      <form onSubmit={(e) => e.preventDefault()}>
        <textarea value={appConfig} onChange={(e) => setAppConfig(e.target.value)} />
        <button onClick={async () => setAppConfig(await invoke('get_config') as string)}>Reload Config</button>
        <button onClick={saveConfig}>Submit</button>
      </form>
      <div>
        <p>Message: {message}</p>
      </div>
    </main>
  );
}

export default App;
