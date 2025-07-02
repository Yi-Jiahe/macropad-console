import { ApplicationConfig, Command } from "./types";
import { useEffect, useState } from "react";
import Button from "@mui/material/Button";
import Dialog from "@mui/material/Dialog";
import DialogTitle from "@mui/material/DialogTitle";
import DialogContent from "@mui/material/DialogContent";
import DialogActions from "@mui/material/DialogActions";
import Box from "@mui/material/Box";
import Breadcrumbs from "@mui/material/Breadcrumbs";
import Typography from "@mui/material/Typography";
import Chip from "@mui/material/Chip";
import CommandElement from "./CommandElement";
import Divider from "@mui/material/Divider";
import List from "@mui/material/List";
import ListItem from "@mui/material/ListItem";

const bindingDividerStyle = {
  paddingTop: 8,
  marginBottom: 8,
  cursor: "pointer",
};

function BindingDivider({ index, hoveredDividerIndex, setHoveredDividerIndex }: { index: number, hoveredDividerIndex: number | null, setHoveredDividerIndex: (index: number | null) => void }) {
  return (
    <Divider
      style={bindingDividerStyle}
      onClick={() => console.log(`Clicked ${index}`)}
      onMouseEnter={() => setHoveredDividerIndex(index)}
      onMouseLeave={() => setHoveredDividerIndex(null)}
      component="li">
      {(hoveredDividerIndex !== null && hoveredDividerIndex === index) &&
        <Chip label="+" style={{ zIndex: 1, position: "absolute", transform: "translate(-50%, -50%)" }} />}
    </Divider>
  );
}

export default function ApplicationProfileElement({
  applicationConfig,
  profileName,
  saveConfig,
  setSelectedProfile
}: {
  applicationConfig: ApplicationConfig,
  profileName: string,
  saveConfig: (configJson: string) => void,
  setSelectedProfile: (profileName: string | null) => void
}) {
  const [openDeleteApplicationProfile, setOpenDeleteApplicationProfile] = useState(false);
  const [hoveredDividerIndex, setHoveredDividerIndex] = useState<number | null>(null);
  const [path, setPath] = useState<Array<Array<number | string>>>([]);
  const [command, setCommand] = useState<Command | null>(null);
  const applicationProfile = applicationConfig.applicationProfiles[profileName];

  const handleCloseDeleteApplicationProfileDialog = () => {
    setOpenDeleteApplicationProfile(false);
  };

  useEffect(() => {
    if (path.length === 0) {
      return;
    }

    let value: any = applicationConfig.applicationProfiles[profileName].bindings;

    for (let i = 0; i < path.length; i++) {
      for (let j = 0; j < path[i].length; j++) {
        const index = path[i][j];
        value = value[index];
        console.log(value);
      }
    }

    setCommand(value);
  }, [path])

  return (
    <Box>
      <Dialog
        open={openDeleteApplicationProfile}
        onClose={handleCloseDeleteApplicationProfileDialog}>
        <DialogTitle>Delete Application Profile</DialogTitle>
        <DialogContent>Are you sure you want to delete the {profileName} profile?</DialogContent>
        <DialogActions>
          <Button variant="text" onClick={() => {
            delete applicationConfig.applicationProfiles[profileName];
            saveConfig(JSON.stringify(applicationConfig));
            setSelectedProfile(null);
            handleCloseDeleteApplicationProfileDialog();
          }}>
            Confirm
          </Button>
          <Button variant="outlined" onClick={handleCloseDeleteApplicationProfileDialog} autoFocus>Cancel</Button>
        </DialogActions>
      </Dialog>
      <Box style={{ display: "flex", justifyContent: "space-between" }}>
        <Breadcrumbs separator=">">
          <Typography onClick={() => setPath([])}>{`${profileName} > bindings`}</Typography>
          {path.map((crumb, index) => (
            <Typography key={index}
              onClick={() => {
                setPath(path.slice(0, index));
                setCommand(null);
              }}>
              {crumb.join(" > ")}
            </Typography>
          ))}
        </Breadcrumbs>
        <Button
          onClick={() => setOpenDeleteApplicationProfile(true)}>
          Delete Profile
        </Button>
      </Box>
      {command === null || path.length === 0 ?
        <Box>
          <Typography variant="h6">Bindings</Typography>
          <Typography variant="body1">
            Bindings are evaluated in the order that they are listed.
          </Typography>
          <List key={profileName}>
            {applicationProfile.bindings.map(([keyCombination, command], index) => {
              return [
                <BindingDivider key={`divider-${index}`} index={index} hoveredDividerIndex={hoveredDividerIndex} setHoveredDividerIndex={setHoveredDividerIndex} />,
                <ListItem key={index}
                  onClick={() => {
                    setPath([[index, 1]]);
                  }}>
                  <KeyCombination KeyCombination={keyCombination} style={{ width: `300px` }} />
                  <Typography variant="body1">{command.displayName}</Typography>
                </ListItem>
              ]
            }
            )}
            <BindingDivider index={applicationProfile.bindings.length} hoveredDividerIndex={hoveredDividerIndex} setHoveredDividerIndex={setHoveredDividerIndex} />
          </List>
        </Box>
        : <Box>
          <KeyCombination KeyCombination={applicationProfile.bindings[path[0][0] as number][0]} />
          <CommandElement command={command} appendPath={(indexes: Array<number | string>) => setPath([...path, indexes])} />
        </Box>}
    </Box>
  );
}

function KeyCombination({ KeyCombination, ...props }: { KeyCombination: string, [key: string]: any }) {
  const keys = KeyCombination.split("+");
  return (
    <Box {...props}>
      {keys.map((key, index) => (
        <Chip key={index} label={key} color={index === keys.length - 1 ? "primary" : "secondary"} />
      ))}
    </Box>
  );
}