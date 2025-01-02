import { Refresh as RefreshIcon } from '@mui/icons-material';
import {
    Alert,
    Box,
    Card,
    CardContent,
    Chip,
    CircularProgress,
    Grid,
    IconButton,
    Tooltip,
    Typography,
} from '@mui/material';
import { useEffect, useState } from 'react';
import { useSystemHealth } from '../hooks/useSystemHealth';

interface HealthMetric {
  label: string;
  value: string | number;
  status: 'healthy' | 'warning' | 'error';
  tooltip?: string;
}

export function SystemHealth() {
  const { health, isLoading, error, refetch } = useSystemHealth();
  const [lastUpdated, setLastUpdated] = useState<Date>(new Date());

  useEffect(() => {
    if (health) {
      setLastUpdated(new Date());
    }
  }, [health]);

  const getStatusColor = (status: string) => {
    switch (status) {
      case 'healthy':
        return 'success';
      case 'warning':
        return 'warning';
      case 'error':
        return 'error';
      default:
        return 'default';
    }
  };

  const metrics: HealthMetric[] = health
    ? [
        {
          label: 'API Status',
          value: health.status,
          status: health.status === 'operational' ? 'healthy' : 'error',
        },
        {
          label: 'Database',
          value: health.services.database,
          status: health.services.database === 'healthy' ? 'healthy' : 'error',
        },
        {
          label: 'Redis',
          value: health.services.redis,
          status: health.services.redis === 'healthy' ? 'healthy' : 'error',
        },
        {
          label: 'Auth Service',
          value: health.services.auth,
          status: health.services.auth === 'healthy' ? 'healthy' : 'error',
        },
        {
          label: 'Uptime',
          value: formatUptime(health.uptime),
          status: 'healthy',
          tooltip: `System has been running since ${new Date(
            Date.now() - health.uptime * 1000
          ).toLocaleString()}`,
        },
      ]
    : [];

  if (isLoading) {
    return (
      <Box display="flex" justifyContent="center" p={3}>
        <CircularProgress />
      </Box>
    );
  }

  if (error) {
    return (
      <Alert severity="error" sx={{ mb: 3 }}>
        Failed to load system health: {error}
      </Alert>
    );
  }

  return (
    <Card>
      <CardContent>
        <Box display="flex" justifyContent="space-between" alignItems="center" mb={2}>
          <Typography variant="h6" component="h2">
            System Health
          </Typography>
          <Box display="flex" alignItems="center" gap={2}>
            <Typography variant="caption" color="textSecondary">
              Last updated: {lastUpdated.toLocaleTimeString()}
            </Typography>
            <IconButton size="small" onClick={() => refetch()}>
              <RefreshIcon />
            </IconButton>
          </Box>
        </Box>

        <Grid container spacing={3}>
          {metrics.map((metric) => (
            <Grid item xs={12} sm={6} md={4} key={metric.label}>
              <Tooltip title={metric.tooltip || ''}>
                <Box
                  sx={{
                    p: 2,
                    border: 1,
                    borderColor: 'divider',
                    borderRadius: 1,
                  }}
                >
                  <Typography variant="body2" color="textSecondary" gutterBottom>
                    {metric.label}
                  </Typography>
                  <Box
                    display="flex"
                    justifyContent="space-between"
                    alignItems="center"
                  >
                    <Typography variant="h6">{metric.value}</Typography>
                    <Chip
                      label={metric.status}
                      color={getStatusColor(metric.status)}
                      size="small"
                    />
                  </Box>
                </Box>
              </Tooltip>
            </Grid>
          ))}
        </Grid>

        {health?.ssl && (
          <Box mt={3}>
            <Typography variant="subtitle2" gutterBottom>
              SSL Certificate
            </Typography>
            <Grid container spacing={2}>
              <Grid item xs={12} sm={6}>
                <Typography variant="body2" color="textSecondary">
                  Expires: {new Date(health.ssl.expires).toLocaleDateString()}
                </Typography>
              </Grid>
              <Grid item xs={12} sm={6}>
                <Typography variant="body2" color="textSecondary">
                  Issuer: {health.ssl.issuer}
                </Typography>
              </Grid>
            </Grid>
          </Box>
        )}
      </CardContent>
    </Card>
  );
}

function formatUptime(seconds: number): string {
  const days = Math.floor(seconds / 86400);
  const hours = Math.floor((seconds % 86400) / 3600);
  const minutes = Math.floor((seconds % 3600) / 60);

  const parts = [];
  if (days > 0) parts.push(`${days}d`);
  if (hours > 0) parts.push(`${hours}h`);
  if (minutes > 0) parts.push(`${minutes}m`);

  return parts.join(' ') ?? '< 1m';
} 