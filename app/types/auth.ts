export interface User {
  id: string;
  name: string;
  email: string;
  avatar?: string;
  role: UserRole;
  organization: Organization;
  permissions: Permission[];
  preferences: UserPreferences;
  created_at: string;
  last_login: string;
}

export type UserRole = 'admin' | 'user' | 'viewer';

export interface Organization {
  id: string;
  name: string;
  plan: SubscriptionPlan;
  features: Feature[];
  limits: ResourceLimits;
  billing_status: BillingStatus;
}

export interface Permission {
  resource: string;
  action: 'create' | 'read' | 'update' | 'delete' | 'execute';
  conditions?: Record<string, any>;
}

export interface UserPreferences {
  theme: 'light' | 'dark' | 'system';
  notifications: NotificationPreferences;
  dashboard_layout?: Record<string, any>;
  timezone: string;
  language: string;
}

export interface NotificationPreferences {
  email: boolean;
  push: boolean;
  training_completed: boolean;
  model_updates: boolean;
  system_updates: boolean;
  billing_alerts: boolean;
}

export interface SubscriptionPlan {
  id: string;
  name: string;
  price: number;
  billing_cycle: 'monthly' | 'annual';
  features: Feature[];
  limits: ResourceLimits;
}

export interface Feature {
  id: string;
  name: string;
  description: string;
  enabled: boolean;
  tier: 'free' | 'pro' | 'enterprise';
}

export interface ResourceLimits {
  models: number;
  requests_per_day: number;
  training_hours_per_month: number;
  storage_gb: number;
  team_members: number;
  concurrent_trainings: number;
}

export interface BillingStatus {
  status: 'active' | 'past_due' | 'canceled' | 'trial';
  trial_ends?: string;
  next_billing_date: string;
  payment_method?: PaymentMethod;
}

export interface PaymentMethod {
  id: string;
  type: 'credit_card' | 'paypal' | 'wire_transfer';
  last4?: string;
  expiry?: string;
  brand?: string;
}

export interface AuthResponse {
  token: string;
  refresh_token: string;
  expires_in: number;
  user: User;
}

export interface LoginRequest {
  email: string;
  password: string;
  remember_me?: boolean;
}

export interface RegisterRequest {
  name: string;
  email: string;
  password: string;
  organization_name: string;
  plan_id?: string;
}

export interface PasswordResetRequest {
  email: string;
}

export interface PasswordUpdateRequest {
  token: string;
  password: string;
  confirm_password: string;
}

export interface MFASetupResponse {
  secret: string;
  qr_code: string;
  backup_codes: string[];
}

export interface MFAVerifyRequest {
  code: string;
  remember_device?: boolean;
} 