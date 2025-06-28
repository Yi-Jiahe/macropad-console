import { ApplicationAction, ApplicationConfig, ApplicationProfile } from "./types";
import { useState } from "react";
import Autocomplete, { createFilterOptions } from "@mui/material/Autocomplete";
import TextField from "@mui/material/TextField";
import Button from "@mui/material/Button";
import Dialog from "@mui/material/Dialog";
import DialogTitle from "@mui/material/DialogTitle";
import DialogContent from "@mui/material/DialogContent";
import DialogActions from "@mui/material/DialogActions";
import Box from "@mui/material/Box";
import Typography from "@mui/material/Typography";

interface ApplicationProfileFilterType {
  profileName?: string;
  inputValue?: string;
  label: string;
}

const filter = createFilterOptions<ApplicationProfileFilterType>();

export default function ApplicationConfigElement({ applicationConfig, saveConfig }: { applicationConfig: ApplicationConfig, saveConfig: (configJson: string) => void }) {
  const [selectedProfile, setSelectedProfile] = useState<string | null>(null);
  const [openDeleteApplicationProfile, setOpenDeleteApplicationProfile] = useState(false);

  const handleCloseDeleteApplicationProfileDialog = () => {
    setOpenDeleteApplicationProfile(false);
  };

  return (
    <>
      <Autocomplete
        value={selectedProfile}
        onChange={async (event, newValue) => {
          console.log("onChange", event, newValue);

          // Selected using Enter
          if (typeof newValue === "string") {
            setSelectedProfile(newValue);
          } else if (newValue && newValue.inputValue) {
            // Create a new application profile
            applicationConfig.applicationProfiles[newValue.inputValue] = {
              actions: [],
            };
            saveConfig(JSON.stringify(applicationConfig));
            setSelectedProfile(newValue.inputValue);
          } else if (newValue && newValue.profileName) {
            setSelectedProfile(newValue.profileName);
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
      {selectedProfile &&
        <>
          <Dialog
            open={openDeleteApplicationProfile}
            onClose={handleCloseDeleteApplicationProfileDialog}>
            <DialogTitle>Delete Application Profile</DialogTitle>
            <DialogContent>Are you sure you want to delete the {selectedProfile} profile?</DialogContent>
            <DialogActions>
              <Button variant="text" onClick={() => {
                delete applicationConfig.applicationProfiles[selectedProfile];
                saveConfig(JSON.stringify(applicationConfig));
                setSelectedProfile(null);
                handleCloseDeleteApplicationProfileDialog();
              }}>
                Confirm
              </Button>
              <Button variant="outlined" onClick={handleCloseDeleteApplicationProfileDialog} autoFocus>Cancel</Button>
            </DialogActions>
          </Dialog>
          <Button
            onClick={() => setOpenDeleteApplicationProfile(true)}>
            Delete Profile
          </Button>
          <>
            {
              Object.entries(applicationConfig.applicationProfiles)
                .filter(([profileName, _]: [string, ApplicationProfile]) => profileName === selectedProfile)
                .map(([profileName, profile]: [string, ApplicationProfile]) => (
                  <Box key={profileName}>
                    <Typography variant="h5">{profileName}</Typography>
                    <table>

                      {profile.actions.map(([action, applicationAction], index) => (
                        <tr key={index}>
                          <td>
                            {action.buttonPress && <>Button Press {action.buttonPress.button}</>}
                            {action.encoderIncrement && <>Encoder Increment {action.encoderIncrement.id}</>}
                            {action.encoderDecrement && <>Encoder Decrement {action.encoderDecrement.id}</>}
                          </td>
                          <td>
                            <ApplicationActionElement applicationAction={applicationAction} />
                          </td>
                        </tr>
                      ))}
                    </table>

                  </Box>
                ))
            }
          </>
        </>}
    </>
  )
}

function ApplicationActionElement({ applicationAction }: { applicationAction: ApplicationAction }) {
  return (
    <>
      {applicationAction.keyTap && <>Key Tap {applicationAction.keyTap.key}</>}
      {applicationAction.keyPress && <>Key Press {applicationAction.keyPress.key}</>}
      {applicationAction.keyRelease && <>Key Release {applicationAction.keyRelease.key}</>}
      {applicationAction.openRadialMenu &&
        <>
          <h3>Open Radial Menu</h3>
          {applicationAction.openRadialMenu.items.map((item, index) => (
            <div key={index}>
              <div>{item.label}</div>
              <ApplicationActionElement applicationAction={item.action} />
            </div>
          ))}
        </>
      }
      {applicationAction.macroTap && <>Macro Tap</>}
    </>
  );
}
