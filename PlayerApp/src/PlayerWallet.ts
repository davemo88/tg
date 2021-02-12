import { NativeModules } from 'react-native';
const { PlayerWalletModule } = NativeModules
interface PlayerWalletInterface {
   call_cli(command: string): string;
   call_cli_with_passphrase(command: string, passphrase: string): string;
}
export default PlayerWalletModule as PlayerWalletInterface;
