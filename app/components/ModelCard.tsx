import {
    Box,
    Button,
    Card,
    CardActions,
    CardContent,
    Chip,
    LinearProgress,
    Tooltip,
    Typography,
} from '@mui/material';
import { Model } from '../types/models';

interface ModelCardProps {
  model: Model;
  selected: boolean;
  onSelect: () => void;
}

export function ModelCard({ model, selected, onSelect }: ModelCardProps) {
  const getStatusColor = (status: string) => {
    switch (status.toLowerCase()) {
      case 'active':
        return 'success';
      case 'training':
        return 'warning';
      case 'error':
        return 'error';
      default:
        return 'default';
    }
  };

  return (
    <Card
      sx={{
        height: '100%',
        display: 'flex',
        flexDirection: 'column',
        border: selected ? '2px solid primary.main' : 'none',
      }}
    >
      <CardContent sx={{ flexGrow: 1 }}>
        <Box display="flex" justifyContent="space-between" alignItems="center" mb={2}>
          <Typography variant="h6" component="h2">
            {model.name}
          </Typography>
          <Chip
            label={model.status}
            color={getStatusColor(model.status)}
            size="small"
          />
        </Box>

        <Typography color="textSecondary" gutterBottom>
          Version: {model.version}
        </Typography>

        <Typography variant="body2" paragraph>
          Task Type: {model.task_type}
        </Typography>

        {model.status === 'training' && (
          <Box mt={2}>
            <Typography variant="body2" color="textSecondary">
              Training Progress
            </Typography>
            <LinearProgress
              variant="determinate"
              value={75}
              sx={{ mt: 1, mb: 1 }}
            />
            <Typography variant="caption" color="textSecondary">
              Estimated completion: 45 minutes
            </Typography>
          </Box>
        )}

        {model.metrics && (
          <Box mt={2}>
            <Typography variant="body2" color="textSecondary" gutterBottom>
              Performance Metrics
            </Typography>
            <Box display="flex" gap={1} flexWrap="wrap">
              {Object.entries(model.metrics).map(([key, value]) => (
                <Tooltip key={key} title={key}>
                  <Chip
                    label={`${key}: ${typeof value === 'number' ? value.toFixed(3) : value}`}
                    size="small"
                    variant="outlined"
                  />
                </Tooltip>
              ))}
            </Box>
          </Box>
        )}
      </CardContent>

      <CardActions>
        <Button size="small" color="primary" onClick={onSelect}>
          Select
        </Button>
        <Button size="small" color="primary" href={`/models/${model.id}`}>
          Details
        </Button>
        {model.status === 'active' && (
          <Button size="small" color="primary" href={`/models/${model.id}/predict`}>
            Use Model
          </Button>
        )}
      </CardActions>
    </Card>
  );
} 