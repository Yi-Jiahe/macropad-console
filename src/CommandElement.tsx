import Box from "@mui/material/Box";
import { Command } from "./types";
import Typography from "@mui/material/Typography";
import Table from "@mui/material/Table";
import Chip from "@mui/material/Chip";
import TableRow from "@mui/material/TableRow";
import { TableCell } from "@mui/material";

export default function CommandElement({ command, appendPath }: { command: Command, appendPath: (index: number) => void }) {
  // TODO: Figure out how to display repeats
  return (
    <Box>
      <Typography variant="h6">{command.displayName}</Typography>
      {command.radialMenuItems &&
        <Table>
          {command.radialMenuItems.map((item, index) => (
            <TableRow key={index} onClick={() => appendPath(index)}>
              <TableCell>{item.label}</TableCell>
              <TableCell>{item.command.displayName}</TableCell>
            </TableRow>
          ))}
        </Table>
      }
      {command.operations &&
        command.operations.map((operation, index) => (
          <Typography key={index}>
            {operation.keyPress && <Chip label={`KeyPress ${operation.keyPress.key}`} />}
            {operation.keyTap && <Chip label={`KeyTap ${operation.keyTap.key}`} />}
            {operation.delay && <Chip label={`Delay ${operation.delay.ms}ms`} />}
            {operation.repeat && <Chip label={`Repeat ${operation.repeat.times} times`} />}
          </Typography>
        ))}
    </Box>
  )
}