import { NativeModules } from 'react-native';
const { PlayerWalletModule } = NativeModules
interface PlayerWalletInterface {
   call_cli(command: string): string;
}
export default PlayerWalletModule as PlayerWalletInterface;
