import { ApplicationConfig } from "./types";
import { useState } from "react";
import Button from "@mui/material/Button";
import Dialog from "@mui/material/Dialog";
import DialogTitle from "@mui/material/DialogTitle";
import DialogContent from "@mui/material/DialogContent";
import DialogActions from "@mui/material/DialogActions";
import Box from "@mui/material/Box";
import Typography from "@mui/material/Typography";
import Table from '@mui/material/Table';
import TableRow from '@mui/material/TableRow';
import TableCell from '@mui/material/TableCell';
import List from "@mui/material/List";
import ListItem from "@mui/material/ListItem";
import ListItemButton from "@mui/material/ListItemButton";

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
  const applicationProfile = applicationConfig.applicationProfiles[profileName];

  const handleCloseDeleteApplicationProfileDialog = () => {
    setOpenDeleteApplicationProfile(false);
  };

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
        <Typography variant="h5">{profileName}</Typography>
        <Button
          onClick={() => setOpenDeleteApplicationProfile(true)}>
          Delete Profile
        </Button>
      </Box>
      <Box style={{ display: "flex" }}>
        <Box style={{ flexGrow: 2 }}>
          <Typography variant="h6">Bindings</Typography>
          <Table key={profileName}>
            {applicationProfile.actions.map(([action, applicationAction], index) => (
              <TableRow key={index}
                hover>
                <TableCell>
                  {action.buttonPress &&
                    <Typography variant="body1">Button Press {action.buttonPress.button}</Typography>}
                  {action.encoderIncrement &&
                    <Typography variant="body1">Encoder Increment {action.encoderIncrement.id}</Typography>}
                  {action.encoderDecrement &&
                    <Typography variant="body1">Encoder Decrement {action.encoderDecrement.id}</Typography>}
                </TableCell>
                <TableCell>
                  {applicationAction.openRadialMenu &&
                    <Typography variant="body1">Radial Menu</Typography>}
                  {applicationAction.macroTap &&
                    <Typography variant="body1">Macro Tap</Typography>}
                  {applicationAction.keyTap &&
                    <Typography variant="body1">Key Tap {applicationAction.keyTap.key}</Typography>}
                </TableCell>
              </TableRow>
            ))}
          </Table>
        </Box>
        <Box style={{ flexGrow: 1 }}>
          <Typography variant="h6">Actions</Typography>
          <List>
            {applicationProfile.actions.map(([_, applicationAction], index) => (
              <ListItem>
                <ListItemButton onClick={() => { }}>
                  {applicationAction.openRadialMenu &&
                    <Typography variant="body1">Radial Menu</Typography>}
                  {applicationAction.macroTap &&
                    <Typography variant="body1">Macro Tap</Typography>}
                  {applicationAction.keyTap &&
                    <Typography variant="body1">Key Tap {applicationAction.keyTap.key}</Typography>}
                </ListItemButton>
              </ListItem>
            ))}
          </List>
        </Box>
      </Box>
    </Box>
  );
}
