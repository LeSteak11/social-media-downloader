// Provider abstraction layer
import { ResolveResult, ProviderType } from '../types';

export interface Provider {
  id: ProviderType;
  matches(url: string): boolean;
  resolve(url: string): Promise<ResolveResult>;
}

export class ProviderRegistry {
  private providers: Provider[] = [];

  register(provider: Provider): void {
    this.providers.push(provider);
  }

  findProvider(url: string): Provider | null {
    return this.providers.find(p => p.matches(url)) || null;
  }

  async resolve(url: string): Promise<ResolveResult> {
    const provider = this.findProvider(url);
    if (!provider) {
      throw new Error('No provider found for this URL');
    }
    return provider.resolve(url);
  }
}

// Global registry instance
export const providerRegistry = new ProviderRegistry();
