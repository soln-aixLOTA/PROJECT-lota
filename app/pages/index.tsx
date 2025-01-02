import {
    Alert,
    Box,
    Button,
    Card,
    CardContent,
    CircularProgress,
    Container,
    Grid,
    Typography,
} from '@mui/material';
import { useState } from 'react';
import { Header } from '../components/Header';
import { ModelCard } from '../components/ModelCard';
import { SystemHealth } from '../components/SystemHealth';
import { useAuth } from '../hooks/useAuth';
import { useModels } from '../hooks/useModels';

export default function Dashboard() {
  const { user, isLoading: authLoading } = useAuth();
  const { models, isLoading: modelsLoading, error: modelsError } = useModels();
  const [selectedModel, setSelectedModel] = useState<string | null>(null);

  if (authLoading) {
    return (
      <Box
        display="flex"
        justifyContent="center"
        alignItems="center"
        minHeight="100vh"
      >
        <CircularProgress />
      </Box>
    );
  }

  if (!user) {
    return (
      <Box
        display="flex"
        justifyContent="center"
        alignItems="center"
        minHeight="100vh"
      >
        <Alert severity="warning">
          Please log in to access the AI platform.
          <Button color="primary" href="/login" sx={{ ml: 2 }}>
            Login
          </Button>
        </Alert>
      </Box>
    );
  }

  return (
    <Box>
      <Header user={user} />
      <Container maxWidth="lg" sx={{ mt: 4, mb: 4 }}>
        <Grid container spacing={3}>
          {/* System Health */}
          <Grid item xs={12}>
            <SystemHealth />
          </Grid>

          {/* Models Section */}
          <Grid item xs={12}>
            <Typography variant="h5" gutterBottom>
              Available Models
            </Typography>
            {modelsLoading ? (
              <CircularProgress />
            ) : modelsError ? (
              <Alert severity="error">{modelsError}</Alert>
            ) : (
              <Grid container spacing={3}>
                {models?.map((model) => (
                  <Grid item xs={12} sm={6} md={4} key={model.id}>
                    <ModelCard
                      model={model}
                      selected={selectedModel === model.id}
                      onSelect={() => setSelectedModel(model.id)}
                    />
                  </Grid>
                ))}
              </Grid>
            )}
          </Grid>

          {/* Quick Actions */}
          <Grid item xs={12}>
            <Card>
              <CardContent>
                <Typography variant="h6" gutterBottom>
                  Quick Actions
                </Typography>
                <Grid container spacing={2}>
                  <Grid item>
                    <Button
                      variant="contained"
                      color="primary"
                      href="/models/train"
                    >
                      Train New Model
                    </Button>
                  </Grid>
                  <Grid item>
                    <Button
                      variant="outlined"
                      color="primary"
                      href="/models/import"
                    >
                      Import Model
                    </Button>
                  </Grid>
                  <Grid item>
                    <Button
                      variant="outlined"
                      color="secondary"
                      href="/documentation"
                    >
                      View Documentation
                    </Button>
                  </Grid>
                </Grid>
              </CardContent>
            </Card>
          </Grid>
        </Grid>
      </Container>
    </Box>
  );
} 