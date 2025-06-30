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
import Chip from "@mui/material/Chip";

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
          <Typography variant="body1">
            Bindings are evaluated in the order that they are listed.
          </Typography>
          <Table key={profileName}>
            {applicationProfile.bindings.map(([action, command], index) => (
              <TableRow key={index}
                hover>
                <TableCell>
                  {action.buttonPress &&
                    <Chip label={`BTN_${action.buttonPress.id}`} />}
                  {action.encoderIncrement &&
                    <Chip label={`ENC_${action.encoderIncrement.id}_INC`} />}
                  {action.encoderDecrement &&
                    <Chip label={`ENC_${action.encoderDecrement.id}_DEC`} />}
                </TableCell>
                <TableCell>
                  <Typography variant="body1">{command.displayName}</Typography>
                </TableCell>
              </TableRow>
            ))}
          </Table>
        </Box>
      </Box>
    </Box>
  );
}
