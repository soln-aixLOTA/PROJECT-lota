import {
    Help as HelpIcon,
    Notifications as NotificationsIcon,
    Settings as SettingsIcon,
} from '@mui/icons-material';
import {
    AppBar,
    Avatar,
    Badge,
    Box,
    Button,
    Divider,
    IconButton,
    Menu,
    MenuItem,
    Toolbar,
    Tooltip,
    Typography,
} from '@mui/material';
import { useState } from 'react';
import { User } from '../types/auth';

interface HeaderProps {
  readonly user: User;
}

export function Header({ user }: Readonly<HeaderProps>) {
  const [anchorEl, setAnchorEl] = useState<null | HTMLElement>(null);
  const [notificationsAnchor, setNotificationsAnchor] = useState<null | HTMLElement>(null);

  const handleMenu = (event: React.MouseEvent<HTMLElement>) => {
    setAnchorEl(event.currentTarget);
  };

  const handleNotifications = (event: React.MouseEvent<HTMLElement>) => {
    setNotificationsAnchor(event.currentTarget);
  };

  const handleClose = () => {
    setAnchorEl(null);
    setNotificationsAnchor(null);
  };

  return (
    <AppBar position="static">
      <Toolbar>
        <Typography variant="h6" component="div" sx={{ flexGrow: 1 }}>
          LotaBots AI Platform
        </Typography>

        <Box sx={{ display: 'flex', alignItems: 'center', gap: 2 }}>
          {/* Navigation Links */}
          <Button color="inherit" href="/dashboard">
            Dashboard
          </Button>
          <Button color="inherit" href="/models">
            Models
          </Button>
          <Button color="inherit" href="/datasets">
            Datasets
          </Button>

          {/* Help */}
          <Tooltip title="Documentation & Help">
            <IconButton color="inherit" href="/documentation">
              <HelpIcon />
            </IconButton>
          </Tooltip>

          {/* Notifications */}
          <Tooltip title="Notifications">
            <IconButton color="inherit" onClick={handleNotifications}>
              <Badge badgeContent={3} color="error">
                <NotificationsIcon />
              </Badge>
            </IconButton>
          </Tooltip>
          <Menu
            anchorEl={notificationsAnchor}
            open={Boolean(notificationsAnchor)}
            onClose={handleClose}
          >
            <MenuItem onClick={handleClose}>
              <Typography variant="body2">
                Model training completed: BERT-base
              </Typography>
            </MenuItem>
            <MenuItem onClick={handleClose}>
              <Typography variant="body2">
                New model version available: GPT-3.5
              </Typography>
            </MenuItem>
            <MenuItem onClick={handleClose}>
              <Typography variant="body2">
                System update scheduled for tomorrow
              </Typography>
            </MenuItem>
            <Divider />
            <MenuItem onClick={handleClose}>
              <Typography variant="body2" color="primary">
                View all notifications
              </Typography>
            </MenuItem>
          </Menu>

          {/* Settings */}
          <Tooltip title="Settings">
            <IconButton color="inherit" href="/settings">
              <SettingsIcon />
            </IconButton>
          </Tooltip>

          {/* User Menu */}
          <Box>
            <Tooltip title="Account settings">
              <IconButton onClick={handleMenu} sx={{ p: 0 }}>
                <Avatar alt={user.name} src={user.avatar}>
                  {user.name[0]}
                </Avatar>
              </IconButton>
            </Tooltip>
            <Menu
              anchorEl={anchorEl}
              open={Boolean(anchorEl)}
              onClose={handleClose}
            >
              <Box sx={{ px: 2, py: 1 }}>
                <Typography variant="subtitle1">{user.name}</Typography>
                <Typography variant="body2" color="textSecondary">
                  {user.email}
                </Typography>
              </Box>
              <Divider />
              <MenuItem onClick={handleClose} component="a" href="/profile">
                Profile
              </MenuItem>
              <MenuItem onClick={handleClose} component="a" href="/billing">
                Billing
              </MenuItem>
              <MenuItem onClick={handleClose} component="a" href="/api-keys">
                API Keys
              </MenuItem>
              <Divider />
              <MenuItem onClick={handleClose} component="a" href="/logout">
                Logout
              </MenuItem>
            </Menu>
          </Box>
        </Box>
      </Toolbar>
    </AppBar>
  );
} 