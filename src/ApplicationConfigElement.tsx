import { ApplicationConfig, ApplicationProfile } from "./types";
import { useEffect, useState } from "react";
import Autocomplete, { createFilterOptions } from "@mui/material/Autocomplete";
import TextField from "@mui/material/TextField";
import Box from "@mui/material/Box";
import ApplicationProfileElement from "./ApplicationProfileElement";

interface ApplicationProfileFilterType {
  profileName?: string;
  inputValue?: string;
  label: string;
}

const filter = createFilterOptions<ApplicationProfileFilterType>();

export default function ApplicationConfigElement({ applicationConfig, saveConfig }: { applicationConfig: ApplicationConfig, saveConfig: (configJson: string) => void }) {
  const [selectedProfileName, setSelectedProfileName] = useState<string | null>(null);

  return (
    <Box>
      <Autocomplete
        value={selectedProfileName}
        onChange={async (event, newValue) => {
          console.log("onChange", event, newValue);

          // Selected using Enter
          if (typeof newValue === "string") {
            setSelectedProfileName(newValue);
          } else if (newValue && newValue.inputValue) {
            // Create a new application profile
            applicationConfig.applicationProfiles[newValue.inputValue] = {
              actions: [],
            };
            saveConfig(JSON.stringify(applicationConfig));
            setSelectedProfileName(newValue.inputValue);
          } else if (newValue && newValue.profileName) {
            setSelectedProfileName(newValue.profileName);
          }
        }}
        // Append a create option to the end of the list
        filterOptions={(options, params) => {
          console.log("filterOptions", options, params);

          const filtered = filter(options, params);

          const { inputValue } = params;
          // Suggest the creation of a new value
          const isExisting = options.some((option) => inputValue === option.profileName);
          if (inputValue !== '' && !isExisting) {
            filtered.push({
              inputValue,
              label: `Create "${inputValue}"`,
            });
          }

          return filtered;
        }}
        options={Object.entries(applicationConfig.applicationProfiles).map(([profileName, _]: [string, ApplicationProfile]): ApplicationProfileFilterType => {
          return { profileName, label: profileName };
        })}
        freeSolo
        renderInput={(params) => <TextField {...params} label="Application Profile" />}
      />
      {selectedProfileName && applicationConfig.applicationProfiles[selectedProfileName] &&
        <ApplicationProfileElement
          applicationConfig={applicationConfig}
          profileName={selectedProfileName}
          saveConfig={saveConfig}
          setSelectedProfile={setSelectedProfileName} />}
    </Box>
  )
}

