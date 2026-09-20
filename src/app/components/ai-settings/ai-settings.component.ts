import { Component, resource } from '@angular/core';
import { MatButtonModule } from '@angular/material/button';
import { MatCardModule } from '@angular/material/card';
import { MatIconModule } from '@angular/material/icon';
import { MatTooltipModule } from '@angular/material/tooltip';
import { RouterLink } from '@angular/router';

import { invoke } from '@tauri-apps/api/core';
import { confirm } from '@tauri-apps/plugin-dialog';

interface McpConfigResponse {
  url: string;
  token: string;
  config: string;
}

@Component({
  imports: [
    MatIconModule,
    MatButtonModule,
    MatCardModule,
    MatTooltipModule,
    RouterLink,
  ],
  selector: 'app-ai-settings',
  styleUrl: './ai-settings.component.scss',
  templateUrl: './ai-settings.component.html',
})
export class AiSettingsComponent {
  protected mcpConfig = resource({
    loader: async () => {
      try {
        const response = await invoke<McpConfigResponse>('get_mcp_config');

        return {
          mcpConfig: response.config,
          mcpUrl: response.url,
        };
      } catch (err) {
        if (err instanceof Error) {
          throw err;
        }
        throw new Error(
          typeof err === 'string' ? err : 'Failed to load MCP configuration',
          { cause: err },
        );
      }
    },
  });

  public async copySettingsToClipboard(): Promise<void> {
    const mcpConfig = this.mcpConfig.value()?.mcpConfig;

    if (mcpConfig) {
      await navigator.clipboard.writeText(mcpConfig);
      return;
    }
  }

  public async enableMcp(): Promise<void> {
    await invoke<McpConfigResponse>('enable_mcp');
    this.mcpConfig.reload();
  }

  public async disableMcp(): Promise<void> {
    const confirmed = await confirm(
      'This will stop the local MCP server. Continue?',
      { title: 'Disable AI access?', kind: 'warning' },
    );

    if (confirmed) {
      await invoke<void>('disable_mcp');
      this.mcpConfig.reload();
    }
  }

  public async regenerateToken(): Promise<void> {
    const confirmed = await confirm(
      'This will invalidate the existing MCP configuration. Continue?',
      { title: 'Regenerate AI access token?', kind: 'warning' },
    );

    if (confirmed) {
      await invoke<McpConfigResponse>('regenerate_mcp_token');
      this.mcpConfig.reload();
    }
  }
}
