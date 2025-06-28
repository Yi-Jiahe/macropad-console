import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from '@tauri-apps/api/event';
import { ActiveWindow, ApplicationConfig } from "./types"
import ApplicationConfigElement from "./ApplicationConfigElement";
import CssBaseline from "@mui/material/CssBaseline";
import Button from "@mui/material/Button";
import Box from "@mui/material/Box";
import Typography from "@mui/material/Typography";


function App() {
  const [activeWindow, setActiveWindow] = useState<ActiveWindow>({
    title: "",
    appName: "",
  });
  const [applicationConfig, setApplicationConfig] = useState<ApplicationConfig | undefined>();
  useEffect(() => {
    listen<ActiveWindow>('active-window-changed', (event) => {
      console.log(event);
      setActiveWindow(event.payload);
    });
  }, [])

  const getConfig = async () => {
    return invoke<string>('get_config').then((configJson) => {
      console.log(configJson);
      setApplicationConfig(JSON.parse(configJson) as ApplicationConfig);
    });
  };

  useEffect(() => {
    getConfig();
  }, [])

  const saveConfig = async () => {
    await invoke('save_config', { configJson: JSON.stringify(applicationConfig) });
    await getConfig();
  };

  return (
    <>
      <CssBaseline />
      <Box>
        <Box>
          <Typography variant="h5">{activeWindow.title}</Typography>
          <Typography variant="body1">{activeWindow.appName}</Typography>
        </Box>
        <Button onClick={getConfig}>Reload Config</Button>
        {applicationConfig && <ApplicationConfigElement applicationConfig={applicationConfig} saveConfig={saveConfig} />}
      </Box>
    </>

  );
}

export default App;
