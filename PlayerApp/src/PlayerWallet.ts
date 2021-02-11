import { NativeModules } from 'react-native';
const { PlayerWalletModule } = NativeModules
interface PlayerWalletInterface {
   call_cli(command: string): string;
   call_cli_bytes(command: number[] ): string;
}
export default PlayerWalletModule as PlayerWalletInterface;
