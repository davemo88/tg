import { NativeModules } from 'react-native';
const { PlayerWalletModule } = NativeModules
interface PlayerWalletInterface {
   call_cli(string): string;
}
export default PlayerWalletModule as PlayerWalletInterface;
